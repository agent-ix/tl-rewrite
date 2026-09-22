//! CI entry point (NFR-004): the program `make guarded-ci`, this repository's
//! README, `CLAUDE.md`, and any hosted workflow dispatch invoke in place of a
//! bare `make ci`.
//!
//! `ci_guard ci` refuses to invoke Make at all if the Makefile text or the
//! calling environment carries a state capable of suppressing
//! prerequisite-failure propagation, then, after Make returns, reconciles
//! the set of gates that actually wrote a completion record against the
//! declared `ci` prerequisite set — regardless of Make's own exit code.
//!
//! `ci_guard record GATE` is the other half: every `ci` prerequisite recipe
//! calls it as the last step of its own recipe, so it only runs on that
//! recipe's own success. Outside a `ci_guard ci` run (e.g. a developer
//! running `make lint` alone) it is a deliberate no-op, not a failure. It
//! also only writes a record for `GATE` when the caller presents the
//! per-gate token Make scoped to `GATE`'s own recipe environment
//! (NFR-004-AC-9, TL-202) — `CI_GUARD_RUN_ID` alone is exported into the
//! whole `make ci` process tree and does not by itself say which gate's
//! recipe a given call came from.
//!
//! See `src/ci_guard.rs` for the checkable logic and
//! `spec/requirements/NFR-004-gate-set-integrity.md` for the requirement.

use std::{
    env, fs,
    path::PathBuf,
    process::{Command, ExitCode},
    time::{SystemTime, UNIX_EPOCH},
};

use sha2::{Digest, Sha256};
use tl_rewrite::ci_guard::{
    authorized_gate, cleanup_gate_tokens, dangerous_makeflags, mint_gate_tokens,
    parse_prerequisites, read_gate_tokens, read_records, reconcile, reset_gates_dir, scan_makefile,
    write_gate_tokens, write_record, GATE_TOKEN_VAR,
};

const RUN_ID_VAR: &str = "CI_GUARD_RUN_ID";
const GATES_DIR: &str = "target/ci-gates";
const MAKE_TARGET: &str = "ci";

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        eprintln!("usage: ci_guard <ci [--dir PATH]|record GATE>");
        return ExitCode::FAILURE;
    };
    match command.as_str() {
        "ci" => run_ci(parse_dir_flag(args)),
        "record" => match args.next() {
            Some(gate) => run_record(&gate),
            None => {
                eprintln!("usage: ci_guard record GATE");
                ExitCode::FAILURE
            }
        },
        other => {
            eprintln!("ci_guard: unknown command {other:?}");
            ExitCode::FAILURE
        }
    }
}

fn parse_dir_flag(mut args: impl Iterator<Item = String>) -> PathBuf {
    while let Some(arg) = args.next() {
        if arg == "--dir" {
            if let Some(dir) = args.next() {
                return PathBuf::from(dir);
            }
        }
    }
    PathBuf::from(".")
}

fn run_record(gate: &str) -> ExitCode {
    let Ok(run_id) = env::var(RUN_ID_VAR) else {
        // Not orchestrated by `ci_guard ci` (e.g. `make lint` run alone for
        // local iteration) — a deliberate no-op, not a failure.
        return ExitCode::SUCCESS;
    };

    // NFR-004-AC-9 (TL-202): a completion record may only be written by (or
    // attributed to) the gate whose own recipe Make scoped this specific
    // token to. `CI_GUARD_RUN_ID` alone — checked above — is not enough: it
    // is exported into the whole `make ci` process tree, so every gate
    // recipe's own subprocess already has it, and before this check any of
    // them could call `record` naming a *different* declared gate than its
    // own. `CI_GUARD_GATE_TOKEN` is not: it only reaches a recipe's
    // environment through the target-specific `export` Make itself applies
    // to that one gate's own recipe.
    let gates_dir = PathBuf::from(GATES_DIR);
    let tokens = read_gate_tokens(&gates_dir);
    let presented = env::var(GATE_TOKEN_VAR).ok();
    if !authorized_gate(&tokens, gate, presented.as_deref()) {
        eprintln!(
            "ci_guard: refusing to record completion for {gate}: no valid per-gate token \
             presented for this run — {GATE_TOKEN_VAR} is scoped by Make to {gate}'s own \
             recipe and is not the same value a different gate's recipe (or a subprocess it \
             spawned) would have inherited"
        );
        return ExitCode::FAILURE;
    }

    match write_record(&gates_dir, gate, &run_id) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("ci_guard: failed to write completion record for {gate}: {err}");
            ExitCode::FAILURE
        }
    }
}

fn run_ci(dir: PathBuf) -> ExitCode {
    let makefile = dir.join("Makefile");

    let violations = scan_makefile(&makefile);
    if !violations.is_empty() {
        eprintln!("ci_guard: refusing to invoke Make — execution-control state detected:");
        for violation in &violations {
            eprintln!("  - {violation}");
        }
        return ExitCode::FAILURE;
    }

    if let Ok(flags) = env::var("MAKEFLAGS") {
        if let Some(reason) = dangerous_makeflags(&flags) {
            eprintln!("ci_guard: refusing to invoke Make — {reason} in the calling environment");
            return ExitCode::FAILURE;
        }
    }

    let text = match fs::read_to_string(&makefile) {
        Ok(text) => text,
        Err(err) => {
            eprintln!("ci_guard: cannot read {}: {err}", makefile.display());
            return ExitCode::FAILURE;
        }
    };
    let Some(declared) = parse_prerequisites(&text, MAKE_TARGET) else {
        eprintln!(
            "ci_guard: no `{MAKE_TARGET}:` rule found in {}",
            makefile.display()
        );
        return ExitCode::FAILURE;
    };

    let gates_dir = dir.join(GATES_DIR);
    if let Err(err) = reset_gates_dir(&gates_dir) {
        eprintln!("ci_guard: cannot reset {}: {err}", gates_dir.display());
        return ExitCode::FAILURE;
    }

    let run_id = fresh_run_id();

    // NFR-004-AC-9 (TL-202): mint a fresh, unpredictable per-gate token for
    // every declared gate and write it where the real Makefile's permanent
    // `-include target/ci-gates/.gate-tokens.mk` line picks it up — Make's
    // own target-specific-variable scoping then delivers each gate's token
    // into only that gate's own recipe environment, not the whole `make ci`
    // process tree the way `CI_GUARD_RUN_ID` below is. `write_gate_tokens`
    // runs after `reset_gates_dir`, so no token left over from (or
    // pre-seeded ahead of) a different run can be mistaken for this one's.
    let tokens = mint_gate_tokens(&declared, &run_id);
    if let Err(err) = write_gate_tokens(&gates_dir, &tokens) {
        eprintln!(
            "ci_guard: cannot write per-gate tokens into {}: {err}",
            gates_dir.display()
        );
        return ExitCode::FAILURE;
    }

    // Explicit, minimal flags: never forward an inherited MAKEFLAGS/MFLAGS
    // into the child, even though the check above should already have
    // refused a dangerous one — defense in depth against a vector this
    // process did not think to check.
    let make_status = match Command::new("make")
        .arg(MAKE_TARGET)
        .current_dir(&dir)
        .env_remove("MAKEFLAGS")
        .env_remove("MFLAGS")
        .env(RUN_ID_VAR, &run_id)
        .status()
    {
        Ok(status) => status,
        Err(err) => {
            eprintln!("ci_guard: failed to invoke make: {err}");
            return ExitCode::FAILURE;
        }
    };

    let records = read_records(&gates_dir);
    let reconciliation = reconcile(&declared, &records, &run_id);

    // Hygiene, not a security boundary: `reset_gates_dir` already wipes any
    // leftover token store at the *start* of the next run, before that run
    // mints its own, so a stale file here cannot be mistaken for a later
    // run's authorization either way.
    cleanup_gate_tokens(&gates_dir);

    if !reconciliation.is_clean() {
        eprintln!("ci_guard: declared/executed gate-set mismatch:");
        for gate in &reconciliation.missing {
            eprintln!("  - missing completion record: {gate}");
        }
        for gate in &reconciliation.unexpected {
            eprintln!("  - completion record for undeclared gate: {gate}");
        }
        return ExitCode::FAILURE;
    }

    if !make_status.success() {
        eprintln!("ci_guard: make {MAKE_TARGET} exited {make_status}");
        return ExitCode::FAILURE;
    }

    println!(
        "ci_guard: all {} declared `{MAKE_TARGET}` gates completed and reconciled",
        declared.len()
    );
    ExitCode::SUCCESS
}

fn fresh_run_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default();
    let pid = std::process::id();
    let mut hasher = Sha256::new();
    hasher.update(nanos.to_le_bytes());
    hasher.update(pid.to_le_bytes());
    hasher
        .finalize()
        .iter()
        .take(16)
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

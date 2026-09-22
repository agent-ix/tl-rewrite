//! Process-level tests for the `ci_guard` CI entry point (NFR-004).
//!
//! These spawn the actual compiled binary against fixture Makefiles rather
//! than calling `src/ci_guard.rs`'s functions directly — the library-level
//! unit tests in that module already exercise the static-inspection,
//! MAKEFLAGS, and reconciliation logic in isolation (TC-057, TC-058, TC-060,
//! TC-061). This file is the integration layer: TC-059 (real recipes write
//! real records), and TC-062/TC-063, which reproduce the exact measurement
//! from Linear TL-64 / `agent-ix/tl-rewrite#11` and its positive control.
//! TC-064 reproduces Linear TL-202 end-to-end: a gate's own recipe forging a
//! completion record for a *different* declared gate using only
//! `CI_GUARD_RUN_ID` and that gate's public name, and confirms the per-gate
//! token binding now refuses it while leaving the legitimate case unaffected.

use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

fn ci_guard_bin() -> &'static str {
    env!("CARGO_BIN_EXE_ci_guard")
}

/// The line every fixture Makefile in this file needs, matching the real
/// Makefile's permanent `-include` (NFR-004-AC-9, TL-202): Make's own
/// target-specific-variable scoping is what delivers each gate's per-run
/// token into only that gate's own recipe environment, so without this line
/// no fixture recipe here would have a token to present and every `record`
/// call would be refused, not just a forged one.
const GATE_TOKENS_INCLUDE: &str = "-include target/ci-gates/.gate-tokens.mk\n";

/// A fixture Makefile with two `ci` prerequisites, `gate-a` and `gate-b`,
/// each running `cmd` and then calling this binary's `record` subcommand —
/// exactly the pattern the real Makefile now uses for all 13 gates.
fn fixture_makefile(cmd: &str, extra_header: &str) -> String {
    let guard = ci_guard_bin();
    format!(
        "{GATE_TOKENS_INCLUDE}{extra_header}.PHONY: ci gate-a gate-b\n\
         ci: gate-a gate-b\n\
         \n\
         gate-a:\n\
         \t{cmd}\n\
         \t\"{guard}\" record gate-a\n\
         \n\
         gate-b:\n\
         \t{cmd}\n\
         \t\"{guard}\" record gate-b\n"
    )
}

fn run_guard(dir: &Path) -> Output {
    Command::new(ci_guard_bin())
        .args(["ci", "--dir"])
        .arg(dir)
        .env_remove("MAKEFLAGS")
        .output()
        .expect("failed to spawn ci_guard")
}

// An unmodified fixture, clean environment, every gate genuinely passing —
// zero exit, no violation.
// Trace: TC-063, NFR-004-AC-7
#[test]
fn clean_makefile_passes() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("Makefile"), fixture_makefile("true", "")).unwrap();

    let output = run_guard(dir.path());
    assert!(
        output.status.success(),
        "expected success, got {:?}\nstdout: {}\nstderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    // Every declared gate has a record from this run.
    let gates_dir = dir.path().join("target/ci-gates");
    assert!(gates_dir.join("gate-a.json").exists());
    assert!(gates_dir.join("gate-b.json").exists());
}

// First reproduction of the tracked measurement: `.IGNORE:` prepended — the
// entry point refuses before Make ever runs, via static inspection, not via
// reconciliation.
// Trace: TC-062, NFR-004-AC-6
#[test]
fn ignore_directive_is_refused_before_make_runs() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("Makefile"),
        fixture_makefile("false", ".IGNORE:\n"),
    )
    .unwrap();

    let output = run_guard(dir.path());
    assert!(!output.status.success(), "expected refusal, got success");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("ignore-directive"),
        "expected the static scan to name ignore-directive, got: {stderr}"
    );

    // Refused before Make ran at all: no completion records exist.
    assert!(!dir.path().join("target/ci-gates/gate-a.json").exists());
}

// Second reproduction of the tracked measurement: a skeleton where every
// recipe is a failing stub, with no execution-control directive present at
// all. Static inspection finds nothing to object to; Make itself stops at
// the first failing gate exactly as it would unguarded. The entry point
// still refuses — this time via reconciliation, because the failing gate
// never wrote a completion record, proving reconciliation is a distinct,
// independently effective mechanism from the static scan (SR-071/SR-074).
// Trace: TC-062, NFR-004-AC-6
#[test]
fn all_failing_recipes_are_refused_via_reconciliation() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("Makefile"), fixture_makefile("false", "")).unwrap();

    let output = run_guard(dir.path());
    assert!(!output.status.success(), "expected refusal, got success");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("missing completion record: gate-a"),
        "expected a reconciliation violation naming gate-a, got: {stderr}"
    );

    // gate-a's own recipe failed before reaching the record line.
    assert!(!dir.path().join("target/ci-gates/gate-a.json").exists());
}

// A real recipe writes its own completion record only on its own successful
// completion — the failing gate above wrote none; this confirms the positive
// half directly, independent of the reconciliation check that consumes it.
// Trace: TC-059, NFR-004-AC-3
#[test]
fn successful_recipe_writes_its_own_record_only() {
    let dir = tempfile::tempdir().unwrap();
    // gate-a succeeds, gate-b fails, using two different fixture commands.
    let guard = ci_guard_bin();
    let makefile = format!(
        "{GATE_TOKENS_INCLUDE}.PHONY: ci gate-a gate-b\n\
         ci: gate-a gate-b\n\
         \n\
         gate-a:\n\
         \ttrue\n\
         \t\"{guard}\" record gate-a\n\
         \n\
         gate-b:\n\
         \tfalse\n\
         \t\"{guard}\" record gate-b\n"
    );
    fs::write(dir.path().join("Makefile"), makefile).unwrap();

    let _output = run_guard(dir.path());
    let gates_dir = dir.path().join("target/ci-gates");
    assert!(
        gates_dir.join("gate-a.json").exists(),
        "gate-a succeeded and must have a record"
    );
    assert!(
        !gates_dir.join("gate-b.json").exists(),
        "gate-b failed before its record line ran and must have none"
    );
}

// Independent second-pass review (SR-079/FND-001), reproduced end-to-end
// against the real compiled binary: a recipe line joining a failing check
// and the record call with a bare `;` used to defeat the entire mechanism
// (static scan, reconciliation, and the trailing raw-exit-code check
// together), because Make's exit status for the line is `ci_guard
// record`'s (always 0), not the check's. Fixed by extending the static
// scan to flag a bare `;` command separator; this test would have observed
// a false success (exit 0) before that fix.
// Trace: TC-057, NFR-004-AC-1
#[test]
fn semicolon_chained_check_and_record_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let guard = ci_guard_bin();
    let makefile = format!(
        ".PHONY: ci gate-a\n\
         ci: gate-a\n\
         \n\
         gate-a:\n\
         \tfalse; \"{guard}\" record gate-a\n"
    );
    fs::write(dir.path().join("Makefile"), makefile).unwrap();

    let output = run_guard(dir.path());
    assert!(
        !output.status.success(),
        "a semicolon-chained check+record line must not report success"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("recipe-chains-commands"),
        "expected the static scan to name recipe-chains-commands, got: {stderr}"
    );

    // Refused before Make ran at all: no completion record exists, so even
    // if the static scan regressed, reconciliation would still have nothing
    // to falsely accept here.
    assert!(!dir.path().join("target/ci-gates/gate-a.json").exists());
}

// TL-202's reproduction, end-to-end: gate-a's own recipe — standing in for
// "any subprocess a gate's recipe spawns" (a compromised/buggy `cargo test`,
// `cargo clippy`, python script, or `quire`/`quoin` invocation) — calls
// `ci_guard record gate-b` using only what it has: `CI_GUARD_RUN_ID`, which
// every recipe in a `ci_guard ci` run shares, and gate-b's name, which is
// public (it is literally in the Makefile). Before NFR-004-AC-9 this alone
// was sufficient to forge gate-b's completion record even though gate-b's
// own recipe never ran. gate-b is deliberately never declared as a `ci`
// prerequisite here (`ci: gate-a` only) so nothing but the forged call could
// ever produce a record naming it — isolating the misattribution from every
// other mechanism this suite already covers (a missing record, a wrong run
// id, Make's own exit code).
// Trace: TC-064, NFR-004-AC-9
#[test]
fn a_gates_own_recipe_cannot_forge_a_record_for_a_different_declared_gate() {
    let dir = tempfile::tempdir().unwrap();
    let guard = ci_guard_bin();
    // Not `ci: gate-a gate-b` — gate-b is intentionally undeclared here so a
    // record naming it can only come from the forgery attempt below, never
    // from a legitimate recipe of its own.
    let makefile = format!(
        "{GATE_TOKENS_INCLUDE}.PHONY: ci gate-a\n\
         ci: gate-a\n\
         \n\
         gate-a:\n\
         \ttrue\n\
         \t\"{guard}\" record gate-a\n\
         \t\"{guard}\" record gate-b\n"
    );
    fs::write(dir.path().join("Makefile"), makefile).unwrap();

    let output = run_guard(dir.path());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("no valid per-gate token presented"),
        "expected the forged gate-b record call to be refused for lacking gate-b's own \
         token, got: {stderr}"
    );

    let gates_dir = dir.path().join("target/ci-gates");
    assert!(
        gates_dir.join("gate-a.json").exists(),
        "gate-a's own, legitimate record call must still succeed"
    );
    assert!(
        !gates_dir.join("gate-b.json").exists(),
        "TL-202: gate-a's recipe must not be able to write a completion record for gate-b — \
         it only has gate-a's own token, not gate-b's"
    );

    // Not just "the run still fails" (gate-b is undeclared here, so
    // reconciliation has nothing to say about it either way, and the run
    // fails only because gate-b is an unexpected record — a different
    // mechanism). The property this test isolates is narrower and sharper:
    // no record naming gate-b exists at all. See
    // `a_forged_record_cannot_survive_as_a_false_pass_for_the_gate_it_names`
    // below for why that narrower property is the one actually load-bearing
    // — reconciliation on its own is *not* a reliable independent catch for
    // a forged record naming a gate that legitimately never completed.
    assert!(!output.status.success());
}

// The precise failure mode NFR-004-AC-9 closes, isolated from every other
// mechanism in this suite: `cargo test`'s own exit status (what gate-a's
// recipe line checks) does not depend on the exit status of every
// subprocess a test may have spawned as a side effect — a fire-and-forget
// or unawaited child's failure is invisible to it. `$(shell ...)` inside a
// recipe line models exactly that: Make evaluates it as a side effect of
// expanding the line's text and never inspects its exit status, so this
// forgery attempt's own failure is invisible to gate-a's recipe exit code
// (confirmed below: `make gate-a` alone, without `ci_guard`, would exit 0
// here) — unlike the sibling test above, where the forged call is a second,
// unhidden recipe line and Make's own raw exit code would have caught the
// resulting failure on its own regardless of NFR-004-AC-9.
//
// Before NFR-004-AC-9, a forged gate-b.json written this way — during
// gate-a's window, before gate-b's own (declared, genuinely failing) recipe
// ever runs — would have been mistaken by `reconcile` for a real, present
// completion, because `reconcile` only checks "does a record naming this
// gate exist for this run id", not "did it come from that gate's own
// recipe". Reconciliation would then stay silent about gate-b specifically,
// masking the mechanism-specific diagnosis NFR-004 exists to provide behind
// a generic "make ci exited 2" from the trailing raw-exit backstop — the
// exact load that backstop was never meant to carry (see this NFR's
// Rationale). NFR-004-AC-9 closes it earlier and more precisely: the forged
// call is refused outright, so no record naming gate-b is ever written, and
// reconciliation correctly names gate-b itself.
// Trace: TC-064, NFR-004-AC-9
#[test]
fn a_forged_record_cannot_survive_as_a_false_pass_for_the_gate_it_names() {
    let dir = tempfile::tempdir().unwrap();
    let guard = ci_guard_bin();
    let makefile = format!(
        "{GATE_TOKENS_INCLUDE}.PHONY: ci gate-a gate-b\n\
         ci: gate-a gate-b\n\
         \n\
         gate-a:\n\
         \ttrue\n\
         \t$(shell \"{guard}\" record gate-b)\n\
         \t\"{guard}\" record gate-a\n\
         \n\
         gate-b:\n\
         \tfalse\n\
         \t\"{guard}\" record gate-b\n"
    );
    fs::write(dir.path().join("Makefile"), makefile).unwrap();

    let output = run_guard(dir.path());
    assert!(
        !output.status.success(),
        "gate-b genuinely never completes its own recipe and must not be reported as a pass"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("missing completion record: gate-b"),
        "reconciliation must name gate-b itself, not fall back to a generic Make exit-code \
         failure that a forged record could otherwise have silenced — got: {stderr}"
    );

    let gates_dir = dir.path().join("target/ci-gates");
    assert!(
        gates_dir.join("gate-a.json").exists(),
        "gate-a's own legitimate record must be unaffected by the forgery attempt it also ran"
    );
    assert!(
        !gates_dir.join("gate-b.json").exists(),
        "no record naming gate-b may exist: it was never written by gate-b's own recipe, and \
         the forgery attempt from gate-a's window must be refused"
    );
}

// The reverse control for the same reproduction: gate-a's own recipe,
// presenting gate-a's own correctly-scoped token for gate-a itself, is
// authorized — the fix must not also break the legitimate case it exists
// alongside.
// Trace: TC-064, NFR-004-AC-9
#[test]
fn a_gates_own_recipe_can_still_record_its_own_completion() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("Makefile"), fixture_makefile("true", "")).unwrap();

    let output = run_guard(dir.path());
    assert!(output.status.success());
    assert!(dir.path().join("target/ci-gates/gate-a.json").exists());
}

// A record file present from an unrelated prior run must not count as a
// pass for the current run — the guard resets the gates directory before
// every invocation (NFR-004-AC-5's defense-in-depth alongside the run-id
// check already unit-tested in src/ci_guard.rs).
// Trace: TC-061, NFR-004-AC-5
#[test]
fn stale_record_from_a_prior_run_does_not_leak_into_a_pass() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("Makefile"), fixture_makefile("false", "")).unwrap();

    let gates_dir = dir.path().join("target/ci-gates");
    fs::create_dir_all(&gates_dir).unwrap();
    fs::write(
        gates_dir.join("gate-a.json"),
        br#"{"gate":"gate-a","run_id":"a-run-from-yesterday"}"#,
    )
    .unwrap();

    let output = run_guard(dir.path());
    assert!(
        !output.status.success(),
        "a stale record must not make a failing run report success"
    );
}

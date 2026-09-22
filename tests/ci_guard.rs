//! Process-level tests for the `ci_guard` CI entry point (NFR-004).
//!
//! These spawn the actual compiled binary against fixture Makefiles rather
//! than calling `src/ci_guard.rs`'s functions directly — the library-level
//! unit tests in that module already exercise the static-inspection,
//! MAKEFLAGS, and reconciliation logic in isolation (TC-057, TC-058, TC-060,
//! TC-061). This file is the integration layer: TC-059 (real recipes write
//! real records), and TC-062/TC-063, which reproduce the exact measurement
//! from Linear TL-64 / `agent-ix/tl-rewrite#11` and its positive control.

use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

fn ci_guard_bin() -> &'static str {
    env!("CARGO_BIN_EXE_ci_guard")
}

/// A fixture Makefile with two `ci` prerequisites, `gate-a` and `gate-b`,
/// each running `cmd` and then calling this binary's `record` subcommand —
/// exactly the pattern the real Makefile now uses for all 13 gates.
fn fixture_makefile(cmd: &str, extra_header: &str) -> String {
    let guard = ci_guard_bin();
    format!(
        "{extra_header}.PHONY: ci gate-a gate-b\n\
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
        ".PHONY: ci gate-a gate-b\n\
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

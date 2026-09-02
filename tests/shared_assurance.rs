//! Tests for the shared assurance intake path (FR-006).
//!
//! These follow this repository's own binding idiom: a `// Trace:` comment above
//! each `#[test]`, which is what Quire's census reads. They invoke the gates
//! rather than reimplementing them, because a test that recomputes what a gate
//! computes is a second implementation that can agree with itself while both are
//! wrong.
//!
//! A missing prerequisite is a failure here, never a skip. A gate that stands
//! down when its dependency is absent reports the same green as one that ran.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use serde_json::Value;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// The interpreter `make assurance-env` builds. Its absence is an error.
fn assurance_python() -> PathBuf {
    let path = std::env::var_os("ASSURANCE_PYTHON")
        .map(PathBuf::from)
        .unwrap_or_else(|| root().join(".venv-assurance/bin/python"));
    assert!(
        path.is_file(),
        "the pinned assurance interpreter is missing at {}. Run `make assurance-env`. \
         This is a failure and not a skip: a gate that stands down when its dependency \
         is absent reports the same green as one that ran.",
        path.display()
    );
    path
}

fn run(program: &Path, arguments: &[&str]) -> (i32, String, String) {
    let output = Command::new(program)
        .args(arguments)
        .current_dir(root())
        .output()
        .unwrap_or_else(|error| panic!("failed to run {}: {error}", program.display()));
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

fn json_gate(program: &Path, arguments: &[&str]) -> Value {
    let (code, stdout, stderr) = run(program, arguments);
    assert_eq!(code, 0, "{arguments:?} exited {code}\n{stdout}\n{stderr}");
    serde_json::from_str(&stdout)
        .unwrap_or_else(|error| panic!("{arguments:?} did not emit JSON: {error}\n{stdout}"))
}

fn head_revision() -> String {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(root())
        .output()
        .expect("git rev-parse failed");
    String::from_utf8_lossy(&output.stdout).trim().to_owned()
}

/// The chain is expensive and several tests read it. It runs once per test
/// binary, and every reader sees the same run rather than a different one.
static CHAIN: OnceLock<Value> = OnceLock::new();

fn chain_report() -> &'static Value {
    CHAIN.get_or_init(|| {
        // The chain runs under the system interpreter: it only shells out to
        // quoin and never imports engineering-assurance.
        let revision = head_revision();
        let (code, stdout, stderr) = run(
            Path::new("python3"),
            &[
                "scripts/assurance_chain.py",
                "--candidate-revision",
                &revision,
                "--json",
            ],
        );
        assert_eq!(code, 0, "the assurance chain exited {code}\n{stderr}");
        serde_json::from_str(&stdout).expect("the assurance chain did not emit JSON")
    })
}

// Trace: TC-023, FR-006-AC-1
#[test]
fn every_shared_pin_is_classified_by_the_packaged_matrix() {
    let python = assurance_python();
    let report = json_gate(&python, &["scripts/check_shared_pins.py", "--json"]);

    let components = report["components"].as_array().expect("components array");
    assert_eq!(
        components.len(),
        4,
        "the matrix pins four components; this run classified {}",
        components.len()
    );
    for component in components {
        assert_eq!(
            component["verdict"], "compatible",
            "{} is {} ({})",
            component["component"], component["verdict"], component["reason"]
        );
    }
    assert_eq!(report["accepted"], true);
    assert!(report["artifact_mismatches"].as_array().unwrap().is_empty());
    assert!(report["mirror_references"].as_array().unwrap().is_empty());

    // Acceptance is reported and never gated on: the pinned release records
    // `pending_human_acceptance` and ships no predicate for it
    // (agent-ix/engineering-assurance#20). Reading an absent field as approval,
    // in either direction, is the mistake this asserts against.
    assert_eq!(report["acceptance_recorded_here"], false);
    assert!(report["acceptance_state"].is_string());

    // The mirror check must be seen to refuse. Without this it is indistinguishable
    // from a check that matches nothing.
    let (code, stdout, stderr) = run(
        &python,
        &[
            "-c",
            "import json,sys;sys.path.insert(0,'scripts');\
             import check_shared_pins as m;\
             pins=json.load(open('assurance/pins.json'));\
             pins['engineering_assurance']['requirement']+=' --registry=https://npm.ix/';\
             print(json.dumps(m.mirror_references(pins)))",
        ],
    );
    assert_eq!(code, 0, "the mirror probe failed: {stderr}");
    let offenders: Vec<String> = serde_json::from_str(stdout.trim()).unwrap();
    assert!(
        !offenders.is_empty(),
        "a mirror registry reference was not detected; the check matches nothing"
    );

    // The consumed-artifact digest check must be seen to refuse. Issue #13
    // deleted the four artifacts this check used to walk — every one of them was
    // read only by the compatibility view — and refilled the list with
    // `engineering_assurance/compatibility.py`, the module `build_report`
    // imports for every component verdict. This probe is what keeps that pin
    // from being decoration.
    let (code, stdout, stderr) = run(
        &python,
        &[
            "-c",
            // Targeted by path, not by position. `[0]` would still have
            // reported a mismatch if the entries were reordered — it would have
            // written a sha256 onto the deliberately undigested matrix entry and
            // caught a file-not-found instead, which is a pass for the wrong
            // reason.
            "import json,sys;sys.path.insert(0,'scripts');\
             import check_shared_pins as m;\
             pins=json.load(open('assurance/pins.json'));\
             hit=[a for a in pins['consumed_artifacts'] \
             if a['path']=='compatibility.py' and 'sha256' in a];\
             assert len(hit)==1, 'compatibility.py is not digest-pinned';\
             hit[0]['sha256']='0'*64;\
             print(json.dumps(m.artifact_digest_mismatches(pins)))",
        ],
    );
    assert_eq!(code, 0, "the consumed-artifact probe failed: {stderr}");
    let problems: Vec<String> = serde_json::from_str(stdout.trim()).unwrap();
    assert!(
        !problems.is_empty(),
        "a changed consumed-artifact digest was not detected; the check matches nothing"
    );

    // And the empty-population branch must fire, because that is the exact shape
    // the deletion would have produced had the list simply been emptied: a
    // re-hash of nothing, reported clean.
    let (code, stdout, stderr) = run(
        &python,
        &[
            "-c",
            "import json,sys;sys.path.insert(0,'scripts');\
             import check_shared_pins as m;\
             pins=json.load(open('assurance/pins.json'));\
             pins['consumed_artifacts']=[a for a in pins['consumed_artifacts'] \
             if 'sha256' not in a];\
             print(json.dumps(m.artifact_digest_mismatches(pins)))",
        ],
    );
    assert_eq!(code, 0, "the empty-population probe failed: {stderr}");
    let vacuous: Vec<String> = serde_json::from_str(stdout.trim()).unwrap();
    assert!(
        vacuous.iter().any(|entry| entry.contains("vacuous")),
        "a consumed-artifact list with no digest in it re-hashed nothing and \
         reported clean: {vacuous:?}"
    );

    // The pin itself must be the live module, not a retained-evidence artifact
    // that nothing opens. Named here so that quietly repointing it at a dead
    // file has to move a literal in this test.
    let pinned = digest_pinned_artifacts();
    assert!(
        pinned.contains("compatibility.py"),
        "assurance/pins.json no longer digest-pins the module check_shared_pins \
         imports for every verdict; the digest check has lost its live subject: \
         {pinned:?}"
    );
}

/// The digest-pinned consumed artifacts, as `assurance/pins.json` declares them.
fn digest_pinned_artifacts() -> BTreeSet<String> {
    let pins: Value = serde_json::from_str(
        &fs::read_to_string(root().join("assurance/pins.json")).expect("assurance/pins.json"),
    )
    .expect("assurance/pins.json is JSON");
    pins["consumed_artifacts"]
        .as_array()
        .expect("consumed_artifacts")
        .iter()
        .filter(|artifact| artifact.get("sha256").is_some())
        .map(|artifact| artifact["path"].as_str().unwrap_or_default().to_owned())
        .collect()
}

// Trace: TC-024, FR-006-AC-2, NFR-003-AC-1, SUITE-004, SUITE-005, SUITE-006, SUITE-007
#[test]
fn the_chain_reaches_quoin_without_quoin_or_quire_executing_a_producer() {
    let report = chain_report();
    assert_eq!(report["matched"], true, "{report:#}");

    for group in ["scenarios", "controls", "adapter_probes"] {
        let items = report[group]
            .as_array()
            .unwrap_or_else(|| panic!("{group}"));
        assert!(!items.is_empty(), "{group} is empty");
        for item in items {
            assert_eq!(
                item["matched"], true,
                "{group} entry did not match: {item:#}"
            );
        }
    }

    // Every attested result is read out of the producer's bytes. Asserting the
    // values here means a chain that reverted to sealing a literal "passed"
    // would still have to agree with what the producers actually wrote — and it
    // gates the DECLARED STATE of every proof, not only the byte identity of
    // what was retained.
    let attested = report["attested_results"]
        .as_object()
        .expect("attested_results");
    assert_eq!(
        attested.len(),
        6,
        "six proof obligations are declared; {} were attested. This was seven \
         until issue #13 removed PROOF-legacy-compatibility with the retained \
         evidence it read.",
        attested.len()
    );
    for (proof, result) in attested {
        assert_eq!(result, "passed", "{proof} was attested {result}");
    }

    // The adapter transcribes one named protocol and refuses another, rather than
    // guessing. A verdict recovered from an unrecognised stream is a verdict
    // recovered from nothing.
    let probes = report["adapter_probes"].as_array().unwrap();
    for required in [
        "refuses-a-foreign-protocol",
        "refuses-an-unnamed-outcome",
        "refuses-an-empty-stream",
        "refuses-a-sibling-producers-stream-under-the-wrong-protocol",
        "accepts-the-real-run",
    ] {
        assert!(
            probes.iter().any(|probe| probe["probe"] == required),
            "adapter probe {required} is missing"
        );
    }
}

/// Write an executable shim for each name that records every invocation.
///
/// The log is the point. A shim that is never consulted and a producer that is
/// never run look identical from the outside, so the shims write down every call
/// and the test reads the file rather than assuming.
///
/// A version query is answered rather than refused, and deliberately so. Asking
/// a tool its version is an observation — it is what the compatibility matrix's
/// own `observe` column does — and it is not the thing this test forbids. What
/// is forbidden is asking a tool to build, compile, test, rewrite, or evaluate
/// anything. Every such invocation is logged and the log must be empty.
///
/// `--version` is matched anywhere in the argv, not just in `$1`, because the
/// MSRV attestation observes `rustup run 1.75.0 cargo --version`: its declared
/// command runs cargo through the pinned toolchain, so the version sealed into
/// the attestation has to come from that toolchain rather than from ambient
/// cargo. That is still a version observation. Anything without a version flag
/// — `cargo build`, `cargo run`, `rustup run … check` — is logged and fails the
/// test, which is what keeps it able to fail.
///
/// `quire` is deliberately NOT in the shim list, and the reason is worth writing
/// down because the first attempt put it there. Shimming `quire` makes this test
/// fail for a reason that is not a boundary violation: `quoin evidence record`
/// itself shells out to `quire coverage` to resolve the obligations an entry
/// binds to. That is Quoin using a static exporter, not anyone executing a
/// producer, and forbidding it would be forbidding the tool from working. The
/// property that actually matters — the driver must not regenerate its own
/// inputs — is asserted separately below by digesting `target/assurance` before
/// and after the run.
///
/// The directory is cleared before each use. Without that, shims written by an
/// earlier run stay on disk, and a change to this helper that stopped writing
/// them would still find the old ones on `PATH` — which is exactly how the
/// "shims absent" probe first came back green.
///
/// Two logs, not one. `invocations.log` records work requests and must be empty;
/// `versions.log` records version queries and must NOT be, because that is the
/// only evidence that the shims were installed and that `PATH` reached them. An
/// adversarial review passed this test with `producer_shims(dir, &[])` — no
/// shims at all — because an empty work log and an absent shim are the same
/// observation until something proves the shim answered.
fn producer_shims(directory: &Path, names: &[&str]) -> (PathBuf, PathBuf) {
    let _ = fs::remove_dir_all(directory);
    fs::create_dir_all(directory).unwrap();
    let log = directory.join("invocations.log");
    let versions = directory.join("versions.log");
    let _ = fs::remove_file(&log);
    let _ = fs::remove_file(&versions);
    for name in names {
        let path = directory.join(name);
        fs::write(
            &path,
            format!(
                "#!/bin/sh\n\
                 for argument in \"$@\"; do\n\
                 case \"$argument\" in\n\
                 --version|-V) echo \"{name} 9.9.9 (shim)\"; \
                 echo \"$0 $@\" >> {}; exit 0 ;;\n\
                 esac\n\
                 done\n\
                 echo \"$0 $@\" >> {}\n\
                 exit 97\n",
                versions.display(),
                log.display()
            ),
        )
        .unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        }
    }
    (log, versions)
}

fn run_chain_with_path(shims: &Path) -> std::process::Output {
    let inherited = std::env::var("PATH").unwrap_or_default();
    let revision = head_revision();
    Command::new("python3")
        .args([
            "scripts/assurance_chain.py",
            "--candidate-revision",
            &revision,
        ])
        .current_dir(root())
        .env("PATH", format!("{}:{inherited}", shims.display()))
        .output()
        .expect("failed to run the assurance chain")
}

// Trace: TC-024, FR-006-AC-2, NFR-003-AC-2
#[test]
fn the_chain_never_executes_a_producer_and_the_probe_can_prove_it() {
    // Two runs, because one proves nothing.
    //
    // Run A replaces every producer — cargo, rustup, rustc — with a stub that
    // logs and fails. The chain must finish, and the log must be empty: not one
    // producer was invoked.
    //
    // Run B is the control. It stubs `quoin`, which the chain is supposed to run,
    // and requires the chain to fail and the log to be non-empty. Without it, an
    // empty log in run A would be equally consistent with PATH never being
    // consulted at all.
    // The other half of "never produces": the driver must not rewrite the files
    // it reads. `quire` is not shimmable for this (see `producer_shims`), so the
    // inputs themselves are digested before and after. A driver that regenerated
    // its own static export, corpus replay or MSRV stream would move a byte here.
    let inputs_before = assurance_input_digests();
    assert!(
        !inputs_before.is_empty(),
        "there are no producer inputs to protect; run `make assurance-inputs`"
    );

    let producers = root().join("target/producer-shims");
    let (producer_log, producer_versions) =
        producer_shims(&producers, &["cargo", "rustup", "rustc"]);
    let output = run_chain_with_path(&producers);
    let logged = fs::read_to_string(&producer_log).unwrap_or_default();
    let versions = fs::read_to_string(&producer_versions).unwrap_or_default();
    assert!(
        output.status.success(),
        "the assurance chain failed with producers stubbed, which means it ran one:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        logged.trim().is_empty(),
        "the assurance driver asked a producer to do work, not just to name its version:\n{logged}"
    );
    // The half that makes the half above mean anything. Without it, removing the
    // shims entirely gives an empty work log and a green test.
    assert!(
        !versions.trim().is_empty(),
        "no shim answered a version query, so the shims were not installed or PATH \
         did not reach them, and the empty invocation log above proves nothing"
    );
    for required in ["cargo", "rustc"] {
        assert!(
            versions.contains(required),
            "the {required} shim was never consulted; the log is:\n{versions}"
        );
    }

    let tools = root().join("target/tool-shims");
    let (tool_log, _) = producer_shims(&tools, &["quoin"]);
    let control = run_chain_with_path(&tools);
    let tool_logged = fs::read_to_string(&tool_log).unwrap_or_default();
    assert!(
        !tool_logged.trim().is_empty(),
        "stubbing quoin produced no invocation, so PATH is not being consulted by \
         the subprocess and the run above proves nothing"
    );
    assert!(
        !control.status.success(),
        "the chain succeeded with quoin stubbed out, so it is not actually using it"
    );

    // The third instrument, and the only one that can see a driver which runs a
    // producer and throws the output away. PATH shims cannot: `quoin evidence
    // record` itself runs `quire coverage`, so shimming `quire` fails the run for
    // a reason that is not a boundary violation. The digest check below cannot
    // either, because a discarded output moves no byte. The driver therefore
    // records every command it executes and refuses anything outside quoin and
    // the declared version observations.
    let report = chain_report();
    assert!(
        report["command_audit_violations"]
            .as_array()
            .unwrap()
            .is_empty(),
        "the driver executed a command it is not permitted to run: {}",
        report["command_audit_violations"]
    );
    let executed = report["executed_commands"].as_array().unwrap();
    assert!(
        executed.len() >= 5,
        "the command audit recorded only {} commands, which is too few to be a census",
        executed.len()
    );
    assert!(
        executed
            .iter()
            .any(|item| item.as_str().unwrap_or_default().starts_with("quoin ")),
        "the command audit recorded no quoin invocation at all: {executed:?}"
    );
    // `make assurance-inputs` invokes five programs — cargo, rustup, quire,
    // python3 and .venv-assurance/bin/python — and the PATH shims above can only
    // cover three of them. `python3` is the interpreter this very driver runs
    // under and `quire` is invoked by Quoin itself, so neither can be shimmed
    // without breaking the run for a reason that is not a boundary violation.
    // The audit closes the remaining two: every program the driver executed must
    // be one of the five it is permitted to name.
    let permitted: BTreeSet<&str> = ["quoin", "quire", "ix-flow", "rustc", "rustup"]
        .into_iter()
        .collect();
    for item in executed {
        let command = item.as_str().unwrap_or_default();
        let program = command.split_whitespace().next().unwrap_or_default();
        assert!(
            permitted.contains(program),
            "the driver executed {program}, which is not one of the five programs it \
             may name; the full command was: {command}"
        );
    }

    let inputs_after = assurance_input_digests();
    assert_eq!(
        inputs_before, inputs_after,
        "the assurance driver rewrote one of the producer outputs it is supposed to \
         only read; a driver that can produce its own inputs can produce a green run \
         out of nothing"
    );
}

/// Digest every producer output the chain consumes.
fn assurance_input_digests() -> Vec<(String, String)> {
    let directory = root().join("target/assurance");
    let Ok(entries) = fs::read_dir(&directory) else {
        return Vec::new();
    };
    let mut digests: Vec<(String, String)> = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .map(|path| {
            let output = Command::new("sha256sum").arg(&path).output().unwrap();
            (
                path.file_name().unwrap().to_string_lossy().into_owned(),
                String::from_utf8_lossy(&output.stdout)
                    .split_whitespace()
                    .next()
                    .unwrap_or_default()
                    .to_owned(),
            )
        })
        .collect();
    digests.sort();
    digests
}

// Trace: TC-025, FR-006-AC-3, SUITE-002, SUITE-003
#[test]
fn the_sealed_records_impact_snapshot_is_the_quire_export() {
    let report = chain_report();
    let export = root().join(report["quire_export"].as_str().expect("quire_export"));
    let bytes =
        fs::read(&export).unwrap_or_else(|error| panic!("{} is absent: {error}", export.display()));

    let digest = {
        let output = Command::new("sha256sum")
            .arg(&export)
            .output()
            .expect("sha256sum failed");
        String::from_utf8_lossy(&output.stdout)
            .split_whitespace()
            .next()
            .expect("sha256sum output")
            .to_owned()
    };
    assert_eq!(
        report["impact_snapshot_digest"], digest,
        "the sealed record's impact snapshot does not name the Quire export it claims"
    );
    // An empty object has a digest too. The snapshot is only worth its content,
    // so the export is required to actually carry the coverage facts the record
    // claims it snapshotted, and to name every requirement this repository has.
    let parsed: Value = serde_json::from_slice(&bytes).expect("the Quire export is JSON");
    let text = String::from_utf8_lossy(&bytes);
    for requirement in [
        "FR-001", "FR-002", "FR-003", "FR-004", "FR-005", "FR-006", "NFR-001", "NFR-002",
        "NFR-003", "StR-001", "StR-002",
    ] {
        assert!(
            text.contains(requirement),
            "the Quire export does not mention {requirement}; it is not a coverage \
             export of this repository"
        );
    }
    assert!(
        parsed.is_object() && !parsed.as_object().unwrap().is_empty(),
        "the Quire export is not a populated document"
    );

    // The measured coverage, pinned. `derive_result` refuses an export that
    // measured nothing or carries a status lie; the figures themselves are
    // asserted here so that an export reporting different totals has to move a
    // number in this file rather than only a threshold in the driver.
    // 68, and the arithmetic is stated so the drop is auditable rather than
    // merely smaller. It was 72 before issue #13, which removed exactly four
    // rows: FR-005-AC-2, FR-006-AC-4, NFR-003-AC-4 and TC-026. Each was a claim
    // about retained evidence that no longer exists, and each went with its test
    // rather than being left to report unbacked.
    let totals = &parsed["totals"];
    assert_eq!(totals["total"], 68, "matrix row count changed: {totals}");
    assert_eq!(
        totals["backed"], 68,
        "backed-row count changed: {totals}. Every row is backed; if that moved, \
         update spec/test-matrix.md deliberately rather than adjusting this assertion."
    );
    // The field that actually moves. An adversarial review measured that
    // repointing one matrix row at nonexistent test cases leaves `totals.backed`
    // at its full count while `unbacked_rows` gains an entry, so the totals alone
    // are not a check. That was measured at 72/72 and the count is 68/68 now;
    // the figure is left out so it does not go stale again.
    assert!(
        parsed["unbacked_rows"].as_array().unwrap().is_empty(),
        "the Quire export names a matrix row backed by nothing: {}",
        parsed["unbacked_rows"]
    );
    // Kept, and known not to fire for the Functional and Stakeholder tables:
    // Quire reports `status-column-matches-nothing` for them because the
    // configured status column and the archetype's asserted columns are mutually
    // exclusive in a validated repository. agent-ix/quire-contract-ir#21.
    assert!(
        parsed["status_lies"].as_array().unwrap().is_empty(),
        "Quire reported a row whose declared status disagrees with its evidence: {}",
        parsed["status_lies"]
    );

    // And the chain must have read it as such rather than as a not-computed run.
    assert_eq!(
        report["attested_results"]["PROOF-quire-static-export"], "passed",
        "the Quire export was attested as {}",
        report["attested_results"]["PROOF-quire-static-export"]
    );
}

// Trace: TC-027, FR-006-AC-5, NFR-003-AC-3
#[test]
fn all_twelve_verification_outcomes_are_demonstrated_and_paired_with_controls() {
    // The twelve states this migration must keep distinguishable, and the gate
    // that owns each. A state nobody demonstrates is a state nobody would notice
    // the loss of.
    //
    // `unsupported` and `malformed` are owned by the rewrite domain here rather
    // than by the compatibility lane: an excluded catalog rule is unsupported,
    // and a document that will not decode is malformed.
    const REQUIRED: [(&str, &str); 12] = [
        ("pass", "chain"),
        ("fail", "chain/counterexample-corpus"),
        ("unavailable", "chain"),
        ("unsupported", "chain/rule-corpus"),
        ("inconclusive", "chain/counterexample-corpus"),
        ("not-computed", "chain"),
        ("malformed", "chain/counterexample-corpus"),
        ("partial", "chain"),
        ("stale", "chain"),
        ("suspect", "chain"),
        ("vacuous", "chain"),
        ("tampered", "chain"),
    ];

    let report = chain_report();

    // Only MEASURED outcomes count: the chain's `states_demonstrated` is built
    // from cases that ran and matched, never from a label.
    //
    // Before issue #13 this set was the union of the chain's states and the
    // compatibility census's `mapped_states`. That union was measured at the
    // pre-deletion tree and the census contributed nothing the chain did not
    // already have: the chain alone demonstrated all twelve — fail,
    // inconclusive, malformed, not-computed, partial, pass, stale, suspect,
    // tampered, unavailable, unsupported and vacuous. The census's own six-word
    // legacy vocabulary intersected this list only at `inconclusive` and
    // `unavailable`, both owned by the chain in the table above. No verification
    // outcome was reachable only through the census, which is why deleting it
    // costs this test no coverage rather than costing it two states quietly.
    let demonstrated: BTreeSet<String> = report["states_demonstrated"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap().to_owned())
        .collect();

    let missing: Vec<&str> = REQUIRED
        .iter()
        .filter(|(state, _)| !demonstrated.contains(*state))
        .map(|(state, _)| *state)
        .collect();
    assert!(
        missing.is_empty(),
        "these verification outcomes were never demonstrated: {missing:?}; \
         demonstrated: {demonstrated:?}"
    );

    // A set that is merely non-empty proves little; the census that used to
    // widen it is gone, so the chain must be seen to carry the full vocabulary
    // on its own rather than to have shrunk quietly to whatever still passes.
    assert_eq!(
        demonstrated.len(),
        REQUIRED.len(),
        "the chain demonstrated {} states for {} required; the compatibility \
         census no longer widens this set and the chain must carry all of them: \
         {demonstrated:?}",
        demonstrated.len(),
        REQUIRED.len()
    );

    // Every negative names the positive control that proves the step it refuses
    // is a step that works.
    let controls = report["controls"].as_array().unwrap();
    assert!(!controls.is_empty(), "no positive controls were run");
    let negatives: BTreeSet<&str> = controls
        .iter()
        .map(|control| control["pairs_with"].as_str().unwrap())
        .collect();
    for required in [
        "retained-bytes-changed-after-sealing",
        "refuse-an-edited-receipt",
        "stale-candidate-binding",
        "attested-failed",
        "excluded-rules-stay-unsupported",
        "counterexamples-are-retained-not-counted",
    ] {
        assert!(
            negatives.contains(required),
            "the negative {required} has no positive control"
        );
    }
}

// Trace: TC-028, FR-006-AC-6, FR-004-AC-1, StR-002-VC-2
#[test]
fn every_counterexample_is_a_replayed_witness_and_never_a_boolean() {
    let report = chain_report();

    // The count comes from the counterexample corpus, so a producer that stopped
    // finding counterexamples cannot also move the number it is checked against.
    let declared = report["declared_counterexample_counts"]["mismatch"]
        .as_u64()
        .unwrap();
    let reported = report["mismatch_rows"].as_u64().unwrap();
    assert_eq!(
        declared, 5,
        "the counterexample corpus declares {declared} mismatch cases; if a case was \
         added or removed this expectation should move deliberately"
    );
    // Every count, not only the one the scenario compares against. The corpus
    // cross-checks its `counts` block against its own `cases` list, which a
    // single coordinated edit moves on both sides; these literals are the part
    // that does not move with it.
    let counts = &report["declared_counterexample_counts"];
    assert_eq!(counts["non_conclusive"], 5, "counts: {counts}");
    assert_eq!(counts["malformed"], 1, "counts: {counts}");
    assert_eq!(counts["total"], 11, "counts: {counts}");
    let rule_counts = &report["declared_rule_counts"];
    assert_eq!(rule_counts["enabled"], 38, "rule counts: {rule_counts}");
    assert_eq!(rule_counts["excluded"], 2, "rule counts: {rule_counts}");
    assert_eq!(rule_counts["total"], 40, "rule counts: {rule_counts}");
    assert_eq!(
        reported, declared,
        "the producer reported {reported} counterexamples for {declared} declared cases"
    );

    // The witnesses themselves, not a count of them. Each carries a trace with at
    // least one instant and two verdicts that differ; a boolean cannot satisfy
    // this and neither can an empty trace.
    let witnesses = report["counterexample_witnesses"].as_array().unwrap();
    assert_eq!(witnesses.len() as u64, declared);
    for witness in witnesses {
        assert!(
            witness["instants"].as_u64().unwrap_or(0) >= 1,
            "{} carries no counterexample instants: {witness:#}",
            witness["symbol"]
        );
        assert!(
            witness["originalVerdict"] != witness["rewrittenVerdict"],
            "{} recorded a witness whose two verdicts agree, so it separates nothing",
            witness["symbol"]
        );
        assert_eq!(
            witness["counterexampleSha256"]
                .as_str()
                .unwrap_or_default()
                .len(),
            64,
            "{} has no digest for the trace it retained",
            witness["symbol"]
        );
    }

    // The three facts the chain asserts, each named, so that dropping any one of
    // them is visible here rather than only inside the driver.
    let scenarios = report["scenarios"].as_array().unwrap();
    for required in [
        "counterexamples-are-retained-not-counted",
        "counterexample-witnesses-are-independently-replayed",
        "counterexamples-survive-into-retained-bytes",
        "bounded-equivalence-is-never-generalized",
        "non-conclusive-reasons-stay-distinct",
    ] {
        let found = scenarios
            .iter()
            .find(|item| item["scenario"] == required)
            .unwrap_or_else(|| panic!("the scenario {required} did not run"));
        assert_eq!(
            found["matched"], true,
            "{required} did not match: {found:#}"
        );
    }

    // A deliberately unsound rewrite does not drag its proof to a failure: the
    // corpus declares the mismatch, so finding it is the obligation being met.
    assert_eq!(
        report["attested_results"]["PROOF-counterexample-evidence"], "passed",
        "the counterevidence proof was attested {}",
        report["attested_results"]["PROOF-counterexample-evidence"]
    );

    // And the producer's own stream still carries the witness, transcribed by the
    // adapter under its own protocol rather than the rule protocol.
    let (code, stdout, stderr) = run(
        Path::new("python3"),
        &[
            "scripts/assurance_chain.py",
            "--adapt",
            "target/assurance/counterexample-evidence.jsonl",
            "--adapt-protocol",
            "tl-rewrite.counterexample-evidence/v1",
        ],
    );
    assert_eq!(code, 0, "the adapter refused the real stream: {stderr}");
    let adapted: Value = serde_json::from_str(&stdout).expect("the adapter emits JSON");
    let carried = adapted["entries"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|entry| entry["domainOutcome"] == "mismatch")
        .count() as u64;
    assert_eq!(
        carried, declared,
        "the adapter dropped the mismatch domain outcome; Quoin's entry vocabulary \
         is three-valued, so the twelve-state word has to survive alongside it"
    );
}

// Trace: TC-029, FR-006-AC-7, SUITE-001
#[test]
fn no_local_evidence_framework_remains_and_no_retained_archive_is_left_behind() {
    let root = root();

    // The generic machinery is gone, by name.
    for removed in [
        "scripts/build_evidence_envelope.py",
        "scripts/collect_evidence.sh",
        "scripts/finalize_collection.py",
        "scripts/verify_evidence.sh",
        "scripts/verify_evidence_manifest.py",
        "scripts/verify_evidence_history.py",
        "scripts/evidence_profile.py",
        "scripts/check_assurance_anchor.py",
        "scripts/check_evidence_root.py",
        "scripts/check_failure_propagation.py",
        "scripts/check_traceability_coverage.py",
        "scripts/check_diff_integrity.py",
        "scripts/check_advisories.py",
        "scripts/check_corpus.py",
        "scripts/run_policy_tests.py",
        "scripts/tool_identity.py",
        "scripts/validate_json_schema.py",
        "scripts/test_evidence_tool.py",
        "scripts/test_evidence_history.py",
        "scripts/test_evidence_profile.py",
        "scripts/test_evidence_root.py",
        "scripts/test_failure_propagation.py",
        "scripts/test_json_schema_gate.py",
        "scripts/test_tool_identity.py",
        "scripts/test_traceability_gate.py",
        "tools.lock",
        "tests/wire_evidence.rs",
        // Issue #13, under the authority of agent-ix/engineering-assurance#7.
        // The retained archive, its only reader, the fixtures that configured it
        // and the two schemas that existed only because the retained envelopes
        // named them by digest. Each schema was proved dead first: neither is
        // reached by include_str! or include_bytes! anywhere in the tree, no
        // module imports jsonschema, and nothing validated any document against
        // either. Two sibling repositories kept schemas that look frozen by name
        // and are live output contracts, so this was measured here rather than
        // inherited.
        "evidence",
        "schemas",
        "scripts/legacy_evidence_view.py",
        "tests/fixtures/legacy-compat",
    ] {
        assert!(
            !root.join(removed).exists(),
            "{removed} is still present; the generic evidence machinery was not removed"
        );
    }

    // The names must be absent from the tree as well as from disk, or a
    // reintroduced reader one directory down would not be caught. The census
    // walks recursively and covers the build and workflow files too. A census
    // this small would be vacuous, so its size is asserted as well.
    // The population is TRACKED FILES, enumerated by Git, not a hand-written
    // directory array. FR-006-AC-7 claims nothing remains "in the repository",
    // and the repository is what is tracked.
    //
    // The array this replaced could not enforce its own completeness. An
    // independent review probed it: deleting `"scripts",` — five files — removed
    // the directory from the walk AND from the guard that was supposed to notice,
    // and the census went green. Deleting `"docs",` did the same. Only a *rename*
    // was caught, which was the one form the guard handled and the one the probe
    // happened to use. Any check whose expected set and observed set are the same
    // literal is decoration; this one is now observed from Git and expected from
    // a separate constant.
    //
    // Reading from Git also fixes three smaller holes at once: `.agent/` is
    // tracked and the array never listed it; untracked scratch files no longer
    // inflate the count and restore the headroom the floor removes; and `.git`,
    // which is a *file* in a linked worktree, is no longer counted.
    //
    // Two enumerations, and the split is deliberate. Moving to `git ls-files`
    // would otherwise have introduced a regression the directory walk did not
    // have: a reintroduced reader sitting UNTRACKED in the working tree would
    // stop being scanned, and "not committed yet" is exactly the state such a
    // file is in while someone is writing it.
    //
    //   * the SCAN covers tracked files plus untracked-but-not-ignored ones, so
    //     a reintroduction is caught before it is ever `git add`ed;
    //   * the COUNT and the area set are tracked-only, so untracked scratch
    //     cannot inflate the population back over the floor or invent an area.
    let git_files = |arguments: &[&str]| -> Vec<String> {
        let output = Command::new("git")
            .args(arguments)
            .current_dir(&root)
            .output()
            .expect("git ls-files failed");
        assert!(
            output.status.success(),
            "git ls-files {arguments:?} exited non-zero; the census cannot \
             enumerate the repository and reporting it clean would be vacuous"
        );
        String::from_utf8_lossy(&output.stdout)
            .split('\0')
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
            .collect()
    };
    // A DENY-list, not an allow-list, and the difference was a real hole. An
    // extension allow-list silently dropped `Makefile` — `extension()` is `None`
    // for it — which is the single worst file to lose here: `compat-view` was a
    // Make target, `COMPAT_RESULT` a Make variable, and the reader was invoked
    // from an `assurance-inputs` recipe line. It was covered at every earlier
    // revision and a probe appending a `compat-view` target to it went green.
    // The same filter dropped extensionless files and `.yaml` (only `.yml` was
    // listed), so a reintroduced reader named `reintroduced_reader` or
    // `reintroduced_reader.yaml` was invisible too.
    //
    // Everything tracked is now scanned except things that cannot carry a
    // reintroduced reader and would only add noise: lockfiles and licence texts.
    // `read_to_string` below already skips anything that is not UTF-8, so binary
    // content needs no rule here.
    let denied = |path: &str| {
        let name = Path::new(path)
            .file_name()
            .and_then(|v| v.to_str())
            .unwrap_or_default();
        name.starts_with("LICENSE") || path.ends_with(".lock")
    };
    let area_of = |path: &str| match path.split_once('/') {
        Some((head, _)) => head.to_owned(),
        None => "<root>".to_owned(),
    };

    let tracked_all = git_files(&["ls-files", "-z"]);
    // Areas come from the UNFILTERED list. Computing them from the filtered one
    // meant a new tracked directory whose files were all filtered out would
    // never appear here, never trip the equality below, and never be scanned.
    let observed_areas: BTreeSet<String> = tracked_all.iter().map(|e| area_of(e)).collect();
    let tracked: Vec<String> = tracked_all
        .iter()
        .filter(|entry| !denied(entry))
        .cloned()
        .collect();

    // A positive control for the untracked half of the scan, written BEFORE the
    // scan is built so that it flows through the real enumeration rather than a
    // parallel one.
    //
    // The first version of this control asserted that a fresh
    // `git ls-files --others` call could see the file. That verified Git works.
    // It did not verify that this census *uses* what Git reports: deleting the
    // untracked loop below left it green, because the control was probing its
    // own call and not the set the scan is built from. The assertion is now
    // against `scanned` itself, which is the only thing the file scan reads.
    //
    // This property has already been lost once — moving the census to
    // `git ls-files` dropped untracked files silently, and it was caught by
    // reading rather than by a gate. It is the only property here with a history
    // of regressing, so it gets a control that runs on every invocation rather
    // than a probe someone has to remember.
    // The control lives in a directory that is itself wholly untracked, because
    // `git ls-files --others` has a `--directory` mode that collapses an
    // untracked directory to its name instead of listing the files inside it. A
    // control sitting directly in a *tracked* directory survives that mode and
    // would report the property intact while a reader dropped into a brand-new
    // directory went unseen. Verified: `--directory` reports
    // `scripts/.census-control/` rather than the file, so the control fails in
    // that mode too.
    //
    // It is nested under `scripts/` rather than placed at the repository root,
    // and that is precautionary rather than a fix for anything. Two tests in this
    // file symlink *every* root entry into a scratch directory and run the chain
    // there, so a control that appears and disappears at the root would be
    // symlinked while it exists and dangle once removed. Nesting keeps the
    // untracked-directory property and takes that hazard away.
    //
    // It is NOT the cause of the intermittent failure in
    // `a_control_naming_a_scenario_that_does_not_exist_is_refused`. That was
    // measured at roughly 2 in 8 with this test filtered out entirely, so it
    // predates this control and is tracked separately as agent-ix/tl-rewrite#15.
    const CONTROL_DIR: &str = "scripts/.census-control";
    const CONTROL: &str = "scripts/.census-control/probe.py";
    let control = root.join(CONTROL);
    let _ = fs::remove_dir_all(root.join(CONTROL_DIR));
    fs::create_dir_all(root.join(CONTROL_DIR)).expect("create control directory");
    fs::write(&control, "# census untracked positive control\n").expect("write control");

    let mut scanned: BTreeSet<String> = tracked.iter().cloned().collect();
    for entry in git_files(&["ls-files", "-z", "--others", "--exclude-standard"]) {
        if !denied(&entry) {
            scanned.insert(entry);
        }
    }

    // Removed before the assertion so a failure cannot leave the tree dirty.
    let _ = fs::remove_dir_all(root.join(CONTROL_DIR));
    assert!(
        scanned.contains(CONTROL),
        "the census did not pick up an untracked file that existed while it \
         enumerated, so the untracked half of the scan is not reaching the set \
         the file scan reads, and a reintroduced reader would stay invisible \
         until someone ran `git add`"
    );
    scanned.remove(CONTROL);

    let sources: Vec<PathBuf> = scanned.iter().map(|entry| root.join(entry)).collect();

    // The expected areas, as a constant separate from what Git reported. A
    // directory that stops being tracked, or a new one that appears and is never
    // inspected, both fail here — and neither can be silenced by editing one
    // list, because the other side comes from Git.
    let expected_areas: BTreeSet<String> = [
        "<root>",
        ".agent",
        ".github",
        "assurance",
        "corpus",
        "docs",
        "examples",
        "scripts",
        "spec",
        "src",
        "tests",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    assert_eq!(
        observed_areas, expected_areas,
        "the set of tracked areas the census inspects has changed. A directory \
         that disappeared here is one the census silently stopped scanning; a \
         directory that appeared is one it has never scanned. Update this \
         constant deliberately."
    );
    // The deleted machinery, by the names a reintroduction would have to use.
    // The two schema filenames are here because an evidence schema reappearing
    // under a different directory is the same defect as the directory coming
    // back.
    const DELETED: [&str; 5] = [
        "legacy_evidence_view",
        "legacy-compat",
        "PROOF-legacy-compatibility",
        "tl-rewrite-evidence-manifest-v1.schema.json",
        "tl-rewrite-evidence-input-v1.schema.json",
    ];

    // Counted over TRACKED files only; the scan below covers more.
    let inspected = tracked.len();
    for path in &sources {
        let Ok(source) = fs::read_to_string(path) else {
            continue;
        };
        // Three files name the deleted machinery on purpose: this test, which
        // asserts its absence; assurance/pins.json, which records what was
        // measured before the deletion; and the change-assurance declaration,
        // which states the constraint the deletion was carried out under.
        // spec/reviews/ and spec/plans/ are dated records of what was found and
        // done at the time; they are not rewritten to un-say it. Everything else
        // must not mention them.
        //
        // Those two directories are permitted for `.md` only, not wholesale. A
        // reintroduced reader dropped into `spec/plans/` as a `.py` or `.rs`
        // file would otherwise be waved through by a rule meant to protect
        // prose, which is the same hole in a different place.
        let relative = path.strip_prefix(&root).unwrap_or(path);
        let relative = relative.to_string_lossy().replace('\\', "/");
        let historical_prose = relative.ends_with(".md")
            && (relative.starts_with("spec/reviews/") || relative.starts_with("spec/plans/"));
        let permitted = matches!(
            relative.as_str(),
            "tests/shared_assurance.rs" | "assurance/pins.json" | "assurance/change-assurance.json"
        ) || historical_prose;
        if permitted {
            continue;
        }
        for name in DELETED {
            assert!(
                !source.contains(name),
                "{} references {name}, which issue #13 deleted; nothing may \
                 reference the removed evidence machinery",
                path.display()
            );
        }
    }
    // Re-derived, and the arithmetic is written out because the previous two
    // attempts at this number were both wrong in ways a stated derivation would
    // have caught.
    //
    // The floor was `> 60` against a population of **87** before issue #13 — 84
    // files across the nine walked directories plus the three named root files —
    // so it carried 26 files of slack, not the 30 an earlier draft of this
    // comment claimed. That draft reached 90 by counting `schemas/`, which was
    // never in the directory array; the claim that it was came from
    // `schemas/README.md`, the same document FND-916 records as describing a
    // census the code had never performed. A rationale anchored on a disproved
    // document is not a rationale.
    //
    // Population now: **93** tracked files — 97 tracked in total, minus the 4 the
    // deny-list drops (`Cargo.lock`, `LICENSE-APACHE`, `LICENSE-MIT` and
    // `corpus/west-v1/LICENSE`). All four are named here, because the previous
    // version of this comment enumerated four exclusions for a count of five and
    // the unnamed one was `Makefile` — the comment was masking the hole rather
    // than describing it.
    //
    // By area: 12 root, 46 `spec`, 9 `tests`, 6 `corpus`, 5 `scripts`, 5 `src`,
    // 3 `assurance`, 3 `examples`, 2 `.github`, 1 `docs`, 1 `.agent`.
    //
    // Derivation, stated so the number is reproducible: the loss this floor must
    // catch is a whole directory going missing, and the largest one a routine
    // change could plausibly shrink without comment is `tests` at 9. `spec` at
    // 46 is larger but only ever grows as reviews land, and growth never trips a
    // lower bound. 93 − 9 = 84, so the floor must be **at least 85** to fail on
    // that loss. 85 is the derived value and is used as-is rather than padded.
    //
    // This number is the coarse instrument. The area-set equality above is what
    // actually catches a directory disappearing, including the small ones —
    // `docs` and `.agent` are one file each and no floor could ever see them go.
    assert!(
        inspected >= 85,
        "the source census inspected {inspected} tracked files, below the derived \
         floor of 85 (population 93, minus `tests` at 9, is 84). The tree shrank \
         substantially. Areas observed: {observed_areas:?}"
    );

    // The Makefile is orchestration, not a trust root, and carries no gate that
    // polices its own execution.
    let makefile = fs::read_to_string(root.join("Makefile")).unwrap();
    for gone in [
        "check-failure-propagation",
        "ci-for-evidence",
        "verify-evidence",
        "evidence-tool",
        "check-tool-identities",
    ] {
        assert!(
            !makefile.contains(gone),
            "the Makefile still carries the {gone} self-attestation target"
        );
    }
    // And the residue is disclosed rather than implied away.
    assert!(
        makefile.contains(".IGNORE:"),
        "the Makefile no longer states what removing the execution-control guard costs"
    );

    // The declared gate set is the gate set. Deleting a prerequisite from `ci:`
    // removes a whole enforcement layer while every remaining test stays green,
    // which is a false green a sibling repository shipped. It is caught here by
    // reading the expanded `ci` prerequisite list rather than by trusting that
    // whoever edited the Makefile also edited a test.
    let expanded = {
        let mut text = String::new();
        let mut lines = makefile
            .lines()
            .skip_while(|line| !line.starts_with("ci: "));
        let mut current = lines
            .next()
            .expect("the Makefile declares a ci target")
            .to_owned();
        loop {
            let trimmed = current.trim_end();
            if let Some(head) = trimmed.strip_suffix('\\') {
                text.push_str(head);
                current = lines.next().expect("a continued ci line ends").to_owned();
                continue;
            }
            text.push_str(trimmed);
            break;
        }
        text
    };
    let declared: BTreeSet<&str> = expanded
        .trim_start_matches("ci:")
        .split_whitespace()
        .collect();
    let required: BTreeSet<&str> = [
        "fmt-check",
        "lint",
        "test",
        "check-corpus",
        "conformance",
        "counterexamples",
        "normalization",
        "deny",
        "audit-unsafe",
        "spec",
        "msrv",
        "rustdoc",
        "assurance",
    ]
    .into_iter()
    .collect();
    assert_eq!(
        declared, required,
        "the `ci` prerequisite set is not the declared gate set; a target was added \
         or removed without saying so here"
    );
}

// Trace: TC-027, FR-006-AC-5, NFR-003-AC-3
#[test]
fn a_control_naming_a_scenario_that_does_not_exist_is_refused() {
    // NFR-003-AC-3 claims this guard is checked. The driver is copied and one
    // `pairs_with` — and only that one — is renamed. Renaming the scenario as
    // well would leave the pairing consistent and prove nothing.
    let scratch = root().join("target/dangling-probe");
    let _ = fs::remove_dir_all(&scratch);
    fs::create_dir_all(scratch.join("scripts")).unwrap();
    let driver = fs::read_to_string(root().join("scripts/assurance_chain.py")).unwrap();

    let control_marker =
        "        \"verify-accepts-an-unedited-receipt\",\n        \"refuse-an-edited-receipt\",";
    assert!(
        driver.contains(control_marker),
        "the control this probe renames is no longer present in the driver"
    );
    let mutated = driver.replacen(
        control_marker,
        "        \"verify-accepts-an-unedited-receipt\",\n        \"refuse-an-edited-receipt-typo\",",
        1,
    );
    assert_ne!(mutated, driver, "the mutation did not apply");
    fs::write(scratch.join("scripts/assurance_chain.py"), &mutated).unwrap();

    // Everything else the driver reads comes from the real tree. Every root entry
    // except `scripts` is symlinked, rather than an enumerated list, so that a
    // driver which starts reading a new directory does not turn this probe into
    // one that fails for an unrelated reason.
    for entry in fs::read_dir(root()).expect("repository root") {
        let path = entry.expect("directory entry").path();
        let name = path
            .file_name()
            .and_then(|v| v.to_str())
            .unwrap_or("")
            .to_owned();
        if name == "scripts" || name == ".git" {
            continue;
        }
        let _ = std::os::unix::fs::symlink(&path, scratch.join(&name));
    }

    let revision = head_revision();
    let output = Command::new("python3")
        .args([
            "scripts/assurance_chain.py",
            "--candidate-revision",
            &revision,
        ])
        .current_dir(&scratch)
        .output()
        .expect("failed to run the mutated chain");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        output.status.code(),
        Some(2),
        "a control naming a non-existent scenario was not refused\n{stderr}"
    );
    assert!(
        stderr.contains("name a scenario that does not exist"),
        "the refusal did not name the cause: {stderr}"
    );
}

// Trace: TC-023, FR-006-AC-1
#[test]
fn the_mirror_scan_refuses_a_registry_reference_in_a_real_file() {
    // The structural branch of `mirror_references` (pins.json) already has a
    // control. The file-scan branch needs its own: without one it is
    // indistinguishable from a loop over files that never match.
    let python = assurance_python();
    let (code, stdout, stderr) = run(
        &python,
        &[
            "-c",
            "import json,sys,pathlib;sys.path.insert(0,'scripts');\
             import check_shared_pins as m;\
             original=pathlib.Path('requirements-assurance.txt').read_text();\
             pathlib.Path('requirements-assurance.txt').write_text(\
             original+'\\n--registry=https://npm.ix/\\n');\
             pins=json.load(open('assurance/pins.json'));\
             found=m.mirror_references(pins);\
             pathlib.Path('requirements-assurance.txt').write_text(original);\
             print(json.dumps(found))",
        ],
    );
    assert_eq!(code, 0, "the mirror file-scan probe failed: {stderr}");
    let offenders: Vec<String> = serde_json::from_str(stdout.trim()).unwrap();
    assert!(
        offenders
            .iter()
            .any(|entry| entry.starts_with("requirements-assurance.txt:")),
        "a mirror reference written into a scanned FILE was not detected; the \
         file-scan branch matches nothing. Detected: {offenders:?}"
    );

    // And the file must be restored, or this test has dirtied the tree.
    let restored = fs::read_to_string(root().join("requirements-assurance.txt")).unwrap();
    assert!(
        !restored.contains("npm.ix/"),
        "the probe left a mirror reference in requirements-assurance.txt"
    );
}

// Trace: TC-030, NFR-002-AC-2, NFR-003-AC-5
#[test]
fn the_published_revision_constants_are_the_resolved_revisions() {
    let report = json_gate(
        Path::new("python3"),
        &["scripts/check_provenance.py", "--json"],
    );
    assert_eq!(report["matched"], true, "{report:#}");
    let entries = report["entries"].as_array().unwrap();
    assert!(
        entries.len() >= 9,
        "the provenance census inspected only {} obligations",
        entries.len()
    );
    for entry in entries {
        assert_eq!(
            entry["outcome"], "pass",
            "{} is {}: {}",
            entry["symbol"], entry["outcome"], entry["detail"]
        );
    }

    // The check must be seen to refuse. `TL_MLTL_REVISION` named a revision the
    // build had not used since the tl-mltl pin moved to merged main, and the only
    // test that touched the field compared the constant to itself, so nothing
    // could see it. This restores the stale value in a scratch copy and requires
    // the census to report a failing row.
    let scratch = root().join("target/provenance-probe");
    let _ = fs::remove_dir_all(&scratch);
    fs::create_dir_all(scratch.join("src")).unwrap();
    fs::create_dir_all(scratch.join("scripts")).unwrap();
    for entry in fs::read_dir(root()).expect("repository root") {
        let path = entry.expect("directory entry").path();
        let name = path
            .file_name()
            .and_then(|v| v.to_str())
            .unwrap_or("")
            .to_owned();
        if name == "src" || name == "scripts" || name == ".git" {
            continue;
        }
        let _ = std::os::unix::fs::symlink(&path, scratch.join(&name));
    }
    fs::copy(
        root().join("scripts/check_provenance.py"),
        scratch.join("scripts/check_provenance.py"),
    )
    .unwrap();
    let library = fs::read_to_string(root().join("src/lib.rs")).unwrap();
    let stale = library.replace(
        "f7eb8bdf93f588050a40b2a4bf7b418f7c63a0e9",
        "fe1c620d7baa743d9c6b4dda27f40d207721fcc9",
    );
    assert_ne!(stale, library, "the probe's mutation did not apply");
    fs::write(scratch.join("src/lib.rs"), stale).unwrap();

    let output = Command::new("python3")
        .args(["scripts/check_provenance.py"])
        .current_dir(&scratch)
        .output()
        .expect("failed to run the mutated provenance check");
    assert_eq!(
        output.status.code(),
        Some(1),
        "a wire constant naming a revision the build never used was not detected:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("dependency:tl-mltl"),
        "the refusal did not name the disagreeing dependency"
    );
}

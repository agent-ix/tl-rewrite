//! Bind the declared and executed `ci` gate set (NFR-004).
//!
//! This module holds the checkable logic behind `src/bin/ci_guard.rs`, the CI
//! entry point invoked in place of a bare `make ci`. It has three jobs:
//!
//! 1. [`scan_makefile`] refuses to proceed if the Makefile text — or, scanned
//!    recursively, any file it `include`s — carries an execution-control
//!    surface capable of suppressing prerequisite-failure propagation.
//! 2. [`dangerous_makeflags`] refuses to proceed if the calling environment's
//!    `MAKEFLAGS` carries the same suppression through a vector static text
//!    inspection cannot see.
//! 3. [`reconcile`], applied to the completion records each `ci` prerequisite
//!    recipe writes only on its own success, binds what was declared to what
//!    actually ran, independent of Make's own exit code.
//!
//! This remediates Linear TL-64 / `agent-ix/tl-rewrite#11`: a single
//! `.IGNORE:` line, or an equivalent execution-control surface, used to make
//! all 13 `ci` prerequisites report success regardless of whether their own
//! recipe failed. See `spec/requirements/NFR-004-gate-set-integrity.md`.

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt, fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

/// Stable class of execution-control surface a [`Violation`] names.
///
/// A closed set, not a message: matching on this rather than comparing
/// strings is how a caller distinguishes one refusal reason from another
/// without re-parsing prose (mirrors [`crate::report::RecordReadErrorCode`]).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ViolationKind {
    /// `.IGNORE:` special target.
    IgnoreDirective,
    /// `.SILENT:` special target.
    SilentDirective,
    /// `.ONESHELL:` special target.
    OneshellDirective,
    /// `.DEFAULT:` special target.
    DefaultDirective,
    /// `SHELL` assignment.
    ShellAssignment,
    /// `.SHELLFLAGS` assignment.
    ShellflagsAssignment,
    /// `MAKEFLAGS` assignment.
    MakeflagsAssignment,
    /// A `-`-prefixed recipe line at the start of a fresh recipe command.
    DashPrefixedRecipe,
    /// A recipe line containing `|| true`.
    RecipeSwallowsFailure,
    /// A recipe line redirecting stderr to `/dev/null`.
    RecipeHidesStderr,
    /// `$(eval` anywhere in the text.
    DynamicEval,
    /// The file (or an `include` target) could not be read.
    Unreadable,
}

impl ViolationKind {
    const fn as_str(self) -> &'static str {
        match self {
            Self::IgnoreDirective => "ignore-directive",
            Self::SilentDirective => "silent-directive",
            Self::OneshellDirective => "oneshell-directive",
            Self::DefaultDirective => "default-directive",
            Self::ShellAssignment => "shell-assignment",
            Self::ShellflagsAssignment => "shellflags-assignment",
            Self::MakeflagsAssignment => "makeflags-assignment",
            Self::DashPrefixedRecipe => "dash-prefixed-recipe",
            Self::RecipeSwallowsFailure => "recipe-swallows-failure",
            Self::RecipeHidesStderr => "recipe-hides-stderr",
            Self::DynamicEval => "dynamic-eval",
            Self::Unreadable => "unreadable",
        }
    }
}

impl fmt::Display for ViolationKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One execution-control surface found while scanning Makefile text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    /// The file the surface was found in (the Makefile, or an `include`d file).
    pub file: PathBuf,
    /// 1-based line number; `0` for a whole-file refusal (e.g. unreadable).
    pub line: usize,
    /// The stable class of surface found.
    pub kind: ViolationKind,
    /// The matched text or a short human-readable explanation.
    pub detail: String,
}

impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}: {} ({})",
            self.file.display(),
            self.line,
            self.kind,
            self.detail
        )
    }
}

const DIRECTIVE_TARGETS: [(&str, ViolationKind); 4] = [
    (".IGNORE:", ViolationKind::IgnoreDirective),
    (".SILENT:", ViolationKind::SilentDirective),
    (".ONESHELL:", ViolationKind::OneshellDirective),
    (".DEFAULT:", ViolationKind::DefaultDirective),
];

const ASSIGNED_VARS: [(&str, ViolationKind); 3] = [
    ("SHELL", ViolationKind::ShellAssignment),
    (".SHELLFLAGS", ViolationKind::ShellflagsAssignment),
    ("MAKEFLAGS", ViolationKind::MakeflagsAssignment),
];

/// Scan `path` (and, recursively, any `include`d file) for the
/// execution-control surfaces NFR-004-AC-1 names: `SHELL`, `.SHELLFLAGS`,
/// `MAKEFLAGS` assignment; `.ONESHELL:`, `.DEFAULT:`, `.IGNORE:`, `.SILENT:`
/// as special targets; a `-`-prefixed recipe line; a recipe containing
/// `|| true` or a stderr-to-`/dev/null` redirect; and `$(eval` anywhere.
///
/// Returns every violation found; an empty result means the text is clean.
/// An unreadable file — including an `include` target that cannot be
/// resolved — is itself a violation rather than a silent skip: this check
/// fails closed on anything it cannot read, exactly as it fails closed on
/// anything it can read and does not like.
pub fn scan_makefile(path: &Path) -> Vec<Violation> {
    let mut visited = BTreeSet::new();
    let mut out = Vec::new();
    scan_file(path, &mut visited, &mut out);
    out
}

fn scan_file(path: &Path, visited: &mut BTreeSet<PathBuf>, out: &mut Vec<Violation>) {
    let canonical = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    if !visited.insert(canonical) {
        return; // already scanned on this walk; avoid an include cycle
    }
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(err) => {
            out.push(Violation {
                file: path.to_path_buf(),
                line: 0,
                kind: ViolationKind::Unreadable,
                detail: format!("cannot read {}: {err}", path.display()),
            });
            return;
        }
    };
    let base_dir = path.parent().unwrap_or_else(|| Path::new("."));
    // Tracks whether the current recipe line is a `\`-continuation of the
    // previous one, since Make's `-`/`@`/`+` prefix only means anything at
    // the start of a fresh recipe command, never on a continuation line
    // (where a leading `-` is just a wrapped command-line flag, e.g.
    // `--manifest ...` on its own wrapped line).
    let mut recipe_continues = false;
    for (idx, line) in text.lines().enumerate() {
        let lineno = idx + 1;
        let trimmed = line.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            recipe_continues = false; // a recipe block cannot span a blank/comment line
            continue;
        }
        if let Some(body) = line.strip_prefix('\t') {
            scan_recipe_line(path, lineno, body, !recipe_continues, out);
            recipe_continues = body.trim_end().ends_with('\\');
        } else {
            recipe_continues = false;
            scan_directive_line(path, lineno, trimmed, base_dir, visited, out);
        }
    }
}

fn scan_recipe_line(
    path: &Path,
    lineno: usize,
    body: &str,
    is_command_start: bool,
    out: &mut Vec<Violation>,
) {
    let mut push = |kind: ViolationKind, detail: String| {
        out.push(Violation {
            file: path.to_path_buf(),
            line: lineno,
            kind,
            detail,
        });
    };
    if is_command_start && body.trim_start().starts_with('-') {
        push(ViolationKind::DashPrefixedRecipe, body.trim().to_string());
    }
    if body.contains("|| true") {
        push(
            ViolationKind::RecipeSwallowsFailure,
            "`|| true` suppresses a non-zero exit".to_string(),
        );
    }
    if body.contains("2>/dev/null") || body.contains("2> /dev/null") {
        push(
            ViolationKind::RecipeHidesStderr,
            "stderr redirected to /dev/null".to_string(),
        );
    }
    if body.contains("$(eval") {
        push(
            ViolationKind::DynamicEval,
            "`$(eval` is refused outright, not analyzed".to_string(),
        );
    }
}

fn scan_directive_line(
    path: &Path,
    lineno: usize,
    trimmed: &str,
    base_dir: &Path,
    visited: &mut BTreeSet<PathBuf>,
    out: &mut Vec<Violation>,
) {
    for (target, kind) in DIRECTIVE_TARGETS {
        if trimmed.starts_with(target) {
            out.push(Violation {
                file: path.to_path_buf(),
                line: lineno,
                kind,
                detail: target.to_string(),
            });
        }
    }
    for (name, kind) in ASSIGNED_VARS {
        if is_assignment(trimmed, name) {
            out.push(Violation {
                file: path.to_path_buf(),
                line: lineno,
                kind,
                detail: trimmed.to_string(),
            });
        }
    }
    if trimmed.contains("$(eval") {
        out.push(Violation {
            file: path.to_path_buf(),
            line: lineno,
            kind: ViolationKind::DynamicEval,
            detail: "`$(eval` is refused outright, not analyzed".to_string(),
        });
    }
    if let Some(target) = include_target(trimmed) {
        let included = base_dir.join(target);
        scan_file(&included, visited, out);
    }
}

fn is_assignment(line: &str, name: &str) -> bool {
    let Some(rest) = line.strip_prefix(name) else {
        return false;
    };
    let rest = rest.trim_start();
    ["=", ":=", "?=", "+=", "!="]
        .iter()
        .any(|op| rest.starts_with(op))
}

fn include_target(line: &str) -> Option<&str> {
    for prefix in ["include ", "-include ", "sinclude "] {
        if let Some(rest) = line.strip_prefix(prefix) {
            return Some(rest.trim());
        }
    }
    None
}

/// Parse the declared prerequisite set of `target`'s rule (e.g. `ci`) from
/// Makefile text, following `\`-continued lines. `None` if the target has no
/// rule head in the text.
pub fn parse_prerequisites(text: &str, target: &str) -> Option<BTreeSet<String>> {
    let head = format!("{target}:");
    let lines: Vec<&str> = text.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        if line.trim_start() == line && line.starts_with(&head) {
            let mut collected = String::new();
            let mut cur = line[head.len()..].to_string();
            loop {
                let continues = cur.trim_end().ends_with('\\');
                let clean = cur.trim_end().trim_end_matches('\\').trim();
                collected.push(' ');
                collected.push_str(clean);
                if !continues {
                    break;
                }
                i += 1;
                if i >= lines.len() {
                    break;
                }
                cur = lines[i].to_string();
            }
            return Some(collected.split_whitespace().map(str::to_string).collect());
        }
        i += 1;
    }
    None
}

/// A flag or bundled short-flag token in `MAKEFLAGS` equivalent to ignoring
/// or continuing past a recipe failure. `-S`/`--no-keep-going`/`--stop`
/// cancel `-k` rather than cause it, and are deliberately not flagged.
///
/// This is a conservative, not exhaustive, parser: it recognizes the
/// documented long/short forms and Make's own bundled-letter form (the
/// space-free token Make itself uses when re-exporting `MAKEFLAGS` to a
/// sub-make, e.g. `ik`). It fails closed on either form rather than trying
/// to parse every possible flag combination — see NFR-004's Scope and
/// SR-075/FND-003 for why this mechanism is expected to need iteration.
pub fn dangerous_makeflags(value: &str) -> Option<String> {
    for token in value.split_whitespace() {
        if token == "-i" || token == "--ignore-errors" {
            return Some(format!("MAKEFLAGS carries {token}"));
        }
        if token == "-k" || token == "--keep-going" {
            return Some(format!("MAKEFLAGS carries {token}"));
        }
        if !token.starts_with('-')
            && !token.contains('=')
            && token.chars().all(|c| c.is_ascii_alphabetic())
            && (token.contains('i') || token.contains('k'))
        {
            return Some(format!(
                "MAKEFLAGS carries a bundled short flag in {token:?}"
            ));
        }
    }
    None
}

/// A completion record one `ci` prerequisite's recipe writes only on its own
/// successful completion (NFR-004-AC-3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GateRecord {
    /// The `ci` prerequisite this record names (e.g. `fmt-check`).
    pub gate: String,
    /// The [`reconcile`] run this record belongs to; a record from a
    /// different run counts as absent (NFR-004-AC-5).
    pub run_id: String,
}

/// Write `gate`'s completion record into `dir`, stamped with `run_id`.
pub fn write_record(dir: &Path, gate: &str, run_id: &str) -> std::io::Result<()> {
    fs::create_dir_all(dir)?;
    let record = GateRecord {
        gate: gate.to_string(),
        run_id: run_id.to_string(),
    };
    let bytes = serde_json::to_vec_pretty(&record)
        .expect("GateRecord serialization is infallible for these field types");
    fs::write(dir.join(format!("{gate}.json")), bytes)
}

/// Read every completion record in `dir`. A directory that does not exist,
/// or a record file that cannot be parsed, contributes no entry — malformed
/// or absent is the same "no record" input to [`reconcile`], never a pass.
pub fn read_records(dir: &Path) -> BTreeMap<String, GateRecord> {
    let mut out = BTreeMap::new();
    let Ok(entries) = fs::read_dir(dir) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let Ok(bytes) = fs::read(&path) else {
            continue;
        };
        let Ok(record) = serde_json::from_slice::<GateRecord>(&bytes) else {
            continue;
        };
        out.insert(record.gate.clone(), record);
    }
    out
}

/// Delete and recreate `dir` so a fresh run starts from no completion
/// records at all — nothing a prior invocation wrote can leak into this
/// run's reconciliation as a false pass.
pub fn reset_gates_dir(dir: &Path) -> std::io::Result<()> {
    if dir.exists() {
        fs::remove_dir_all(dir)?;
    }
    fs::create_dir_all(dir)
}

/// The result of comparing a declared prerequisite set to the completion
/// records actually observed for `run_id` (NFR-004-AC-4/AC-5).
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Reconciliation {
    /// Declared gates with no valid record for this run.
    pub missing: Vec<String>,
    /// Records for this run naming a gate that was not declared.
    pub unexpected: Vec<String>,
}

impl Reconciliation {
    pub fn is_clean(&self) -> bool {
        self.missing.is_empty() && self.unexpected.is_empty()
    }
}

/// Bind `declared` to `records`. A record only counts if its `run_id`
/// matches the run under evaluation — a record left over from, replayed
/// from, or backdated to a different run is treated as no record at all,
/// not a stale-but-valid pass (NFR-004-AC-5).
pub fn reconcile(
    declared: &BTreeSet<String>,
    records: &BTreeMap<String, GateRecord>,
    run_id: &str,
) -> Reconciliation {
    let mut result = Reconciliation::default();
    for gate in declared {
        match records.get(gate) {
            Some(record) if record.run_id == run_id => {}
            _ => result.missing.push(gate.clone()),
        }
    }
    for (gate, record) in records {
        if record.run_id == run_id && !declared.contains(gate) {
            result.unexpected.push(gate.clone());
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_temp(dir: &Path, name: &str, contents: &str) -> PathBuf {
        let path = dir.join(name);
        let mut f = fs::File::create(&path).unwrap();
        f.write_all(contents.as_bytes()).unwrap();
        path
    }

    // Trace: TC-057, NFR-004-AC-1
    #[test]
    fn scan_clean_control_has_no_violations() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp(
            dir.path(),
            "Makefile",
            "CARGO ?= cargo\n\n.PHONY: ci\nci: fmt-check\n\nfmt-check:\n\t$(CARGO) fmt --all -- --check\n",
        );
        assert!(scan_makefile(&path).is_empty());
    }

    // Trace: TC-057, NFR-004-AC-1
    #[test]
    fn scan_detects_ignore_directive() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp(dir.path(), "Makefile", ".IGNORE:\nci:\n\tfalse\n");
        let violations = scan_makefile(&path);
        assert!(violations
            .iter()
            .any(|v| v.kind == ViolationKind::IgnoreDirective));
    }

    // Trace: TC-057, NFR-004-AC-1
    #[test]
    fn scan_detects_silent_directive() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp(dir.path(), "Makefile", ".SILENT:\nci:\n\tfalse\n");
        assert!(scan_makefile(&path)
            .iter()
            .any(|v| v.kind == ViolationKind::SilentDirective));
    }

    // Trace: TC-057, NFR-004-AC-1
    #[test]
    fn scan_detects_oneshell_directive() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp(dir.path(), "Makefile", ".ONESHELL:\nci:\n\tfalse\n");
        assert!(scan_makefile(&path)
            .iter()
            .any(|v| v.kind == ViolationKind::OneshellDirective));
    }

    // Trace: TC-057, NFR-004-AC-1
    #[test]
    fn scan_detects_default_directive() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp(dir.path(), "Makefile", ".DEFAULT:\n\t@true\nci:\n\tfalse\n");
        assert!(scan_makefile(&path)
            .iter()
            .any(|v| v.kind == ViolationKind::DefaultDirective));
    }

    // Trace: TC-057, NFR-004-AC-1
    #[test]
    fn scan_detects_shell_assignment() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp(
            dir.path(),
            "Makefile",
            "SHELL := /bin/false\nci:\n\tfalse\n",
        );
        assert!(scan_makefile(&path)
            .iter()
            .any(|v| v.kind == ViolationKind::ShellAssignment));
    }

    // Trace: TC-057, NFR-004-AC-1
    #[test]
    fn scan_detects_shellflags_assignment() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp(dir.path(), "Makefile", ".SHELLFLAGS := -ec\nci:\n\tfalse\n");
        assert!(scan_makefile(&path)
            .iter()
            .any(|v| v.kind == ViolationKind::ShellflagsAssignment));
    }

    // Trace: TC-057, NFR-004-AC-1
    #[test]
    fn scan_detects_makeflags_assignment() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp(dir.path(), "Makefile", "MAKEFLAGS += -i\nci:\n\tfalse\n");
        assert!(scan_makefile(&path)
            .iter()
            .any(|v| v.kind == ViolationKind::MakeflagsAssignment));
    }

    // Trace: TC-057, NFR-004-AC-1
    #[test]
    fn scan_detects_dash_prefixed_recipe() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp(dir.path(), "Makefile", "ci:\n\t-false\n");
        assert!(scan_makefile(&path)
            .iter()
            .any(|v| v.kind == ViolationKind::DashPrefixedRecipe));
    }

    // Trace: TC-057, NFR-004-AC-1
    // Regression: a `\`-continued recipe line whose wrapped text happens to
    // start with `--flag` is not the Make error-ignoring `-` prefix and must
    // not be flagged. Caught by running the real entry point against this
    // repository's own Makefile (`--manifest $(...)` wrapped onto a
    // continuation line), which is exactly the dogfooding Task-006 exists to do.
    #[test]
    fn scan_does_not_flag_a_wrapped_flag_on_a_continuation_line() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp(
            dir.path(),
            "Makefile",
            "ci:\n\tcargo run --quiet --example x -- \\\n\t\t--manifest corpus/x.json\n",
        );
        assert!(scan_makefile(&path).is_empty());
    }

    // Trace: TC-057, NFR-004-AC-1
    // The actual `-` prefix is still caught even when the same recipe line
    // itself continues onto a further line.
    #[test]
    fn scan_still_flags_dash_prefix_on_first_line_of_a_continued_recipe() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp(dir.path(), "Makefile", "ci:\n\t-false \\\n\t\t--flag\n");
        assert!(scan_makefile(&path)
            .iter()
            .any(|v| v.kind == ViolationKind::DashPrefixedRecipe));
    }

    // Trace: TC-057, NFR-004-AC-1
    #[test]
    fn scan_detects_recipe_swallowing_failure() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp(dir.path(), "Makefile", "ci:\n\tfalse || true\n");
        assert!(scan_makefile(&path)
            .iter()
            .any(|v| v.kind == ViolationKind::RecipeSwallowsFailure));
    }

    // Trace: TC-057, NFR-004-AC-1
    #[test]
    fn scan_detects_stderr_to_null() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp(dir.path(), "Makefile", "ci:\n\tfalse 2>/dev/null\n");
        assert!(scan_makefile(&path)
            .iter()
            .any(|v| v.kind == ViolationKind::RecipeHidesStderr));
    }

    // Trace: TC-057, NFR-004-AC-1
    #[test]
    fn scan_detects_dynamic_eval() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp(dir.path(), "Makefile", "$(eval .IGNORE:)\nci:\n\tfalse\n");
        assert!(scan_makefile(&path)
            .iter()
            .any(|v| v.kind == ViolationKind::DynamicEval));
    }

    // Trace: TC-057, NFR-004-AC-1
    #[test]
    fn scan_recurses_into_include_and_flags_it() {
        let dir = tempfile::tempdir().unwrap();
        write_temp(dir.path(), "extra.mk", ".IGNORE:\n");
        let path = write_temp(dir.path(), "Makefile", "include extra.mk\nci:\n\tfalse\n");
        assert!(scan_makefile(&path)
            .iter()
            .any(|v| v.kind == ViolationKind::IgnoreDirective && v.file.ends_with("extra.mk")));
    }

    // Trace: TC-057, NFR-004-AC-1
    #[test]
    fn scan_flags_unresolvable_include_as_a_violation() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp(
            dir.path(),
            "Makefile",
            "include does-not-exist.mk\nci:\n\tfalse\n",
        );
        assert!(scan_makefile(&path)
            .iter()
            .any(|v| v.kind == ViolationKind::Unreadable));
    }

    // Trace: TC-057, NFR-004-AC-1
    #[test]
    fn scan_ignores_directives_named_only_in_comments() {
        // The Makefile's own header prose discusses .IGNORE:, $(eval), and
        // include as documentation; scanning must not self-trigger on it.
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp(
            dir.path(),
            "Makefile",
            "# .IGNORE: .SILENT: MAKEFLAGS := -i $(eval foo) include x.mk\nci:\n\tfalse\n",
        );
        assert!(scan_makefile(&path).is_empty());
    }

    // Trace: TC-060, NFR-004-AC-4
    #[test]
    fn parse_prerequisites_handles_continuation_lines() {
        let text = "ci: fmt-check lint \\\n\tdeny audit-unsafe\n";
        let declared = parse_prerequisites(text, "ci").unwrap();
        let expected: BTreeSet<String> = ["fmt-check", "lint", "deny", "audit-unsafe"]
            .into_iter()
            .map(String::from)
            .collect();
        assert_eq!(declared, expected);
    }

    // Trace: TC-060, NFR-004-AC-4
    #[test]
    fn parse_prerequisites_returns_none_when_target_absent() {
        assert!(parse_prerequisites("fmt-check:\n\tcargo fmt\n", "ci").is_none());
    }

    // Trace: TC-058, NFR-004-AC-2
    #[test]
    fn dangerous_makeflags_detects_dashed_ignore_errors() {
        assert!(dangerous_makeflags("-i").is_some());
    }

    // Trace: TC-058, NFR-004-AC-2
    #[test]
    fn dangerous_makeflags_detects_dashed_keep_going() {
        assert!(dangerous_makeflags("-k").is_some());
    }

    // Trace: TC-058, NFR-004-AC-2
    #[test]
    fn dangerous_makeflags_detects_bundled_short_flags() {
        assert!(dangerous_makeflags("ik").is_some());
        assert!(dangerous_makeflags("wik").is_some());
    }

    // Trace: TC-058, NFR-004-AC-2
    #[test]
    fn dangerous_makeflags_allows_clean_flags() {
        assert!(dangerous_makeflags("").is_none());
        assert!(dangerous_makeflags("w").is_none());
        assert!(dangerous_makeflags("--no-print-directory").is_none());
    }

    // Trace: TC-058, NFR-004-AC-2
    #[test]
    fn dangerous_makeflags_does_not_flag_stop_negation() {
        // -S/--no-keep-going CANCELS -k; it must never itself be flagged.
        assert!(dangerous_makeflags("-S").is_none());
        assert!(dangerous_makeflags("--no-keep-going").is_none());
    }

    // Trace: TC-060, NFR-004-AC-4
    #[test]
    fn reconcile_reports_missing_and_unexpected() {
        let declared: BTreeSet<String> = ["a", "b"].into_iter().map(String::from).collect();
        let mut records = BTreeMap::new();
        records.insert(
            "a".to_string(),
            GateRecord {
                gate: "a".to_string(),
                run_id: "run-1".to_string(),
            },
        );
        records.insert(
            "c".to_string(),
            GateRecord {
                gate: "c".to_string(),
                run_id: "run-1".to_string(),
            },
        );
        let result = reconcile(&declared, &records, "run-1");
        assert_eq!(result.missing, vec!["b".to_string()]);
        assert_eq!(result.unexpected, vec!["c".to_string()]);
        assert!(!result.is_clean());
    }

    // Trace: TC-060, NFR-004-AC-4
    #[test]
    fn reconcile_is_clean_on_exact_match() {
        let declared: BTreeSet<String> = ["a", "b"].into_iter().map(String::from).collect();
        let mut records = BTreeMap::new();
        for gate in ["a", "b"] {
            records.insert(
                gate.to_string(),
                GateRecord {
                    gate: gate.to_string(),
                    run_id: "run-1".to_string(),
                },
            );
        }
        assert!(reconcile(&declared, &records, "run-1").is_clean());
    }

    // Trace: TC-061, NFR-004-AC-5
    #[test]
    fn reconcile_treats_wrong_run_id_record_as_absent() {
        // A record left over from (or replayed/backdated from) a different
        // run must not count as a pass for the current run (NFR-004-AC-5).
        let declared: BTreeSet<String> = ["a"].into_iter().map(String::from).collect();
        let mut records = BTreeMap::new();
        records.insert(
            "a".to_string(),
            GateRecord {
                gate: "a".to_string(),
                run_id: "stale-run".to_string(),
            },
        );
        let result = reconcile(&declared, &records, "current-run");
        assert_eq!(result.missing, vec!["a".to_string()]);
        assert!(result.unexpected.is_empty());
    }

    // Trace: TC-059, NFR-004-AC-3
    #[test]
    fn read_records_skips_malformed_and_missing_directory() {
        let dir = tempfile::tempdir().unwrap();
        assert!(read_records(dir.path().join("absent").as_path()).is_empty());

        let records_dir = dir.path().join("records");
        fs::create_dir_all(&records_dir).unwrap();
        fs::write(records_dir.join("broken.json"), b"not json").unwrap();
        assert!(read_records(&records_dir).is_empty());
    }

    // Trace: TC-059, NFR-004-AC-3
    #[test]
    fn write_then_read_round_trips() {
        let dir = tempfile::tempdir().unwrap();
        write_record(dir.path(), "fmt-check", "run-1").unwrap();
        let records = read_records(dir.path());
        let record = records.get("fmt-check").unwrap();
        assert_eq!(record.run_id, "run-1");
    }

    // Trace: TC-061, NFR-004-AC-5
    #[test]
    fn reset_gates_dir_clears_prior_contents() {
        let dir = tempfile::tempdir().unwrap();
        let gates = dir.path().join("gates");
        write_record(&gates, "stale", "old-run").unwrap();
        reset_gates_dir(&gates).unwrap();
        assert!(read_records(&gates).is_empty());
    }
}

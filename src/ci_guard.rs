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
//! 4. [`mint_gate_tokens`]/[`authorized_gate`] bind each completion record to
//!    the specific gate recipe Make itself scoped a fresh, per-run token to,
//!    so a subprocess sharing `CI_GUARD_RUN_ID` cannot write a record for a
//!    *different* declared gate than the one whose recipe actually spawned
//!    it (NFR-004-AC-9, TL-202) merely by inheriting that run id.
//!
//! This remediates Linear TL-64 / `agent-ix/tl-rewrite#11`: a single
//! `.IGNORE:` line, or an equivalent execution-control surface, used to make
//! all 13 `ci` prerequisites report success regardless of whether their own
//! recipe failed. See `spec/requirements/NFR-004-gate-set-integrity.md`.

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt, fs,
    io::Read,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

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
    /// A recipe line joins two shell commands with a bare `;`, so Make's
    /// exit status for the line is the *last* command's, not the check's.
    RecipeChainsCommands,
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
            Self::RecipeChainsCommands => "recipe-chains-commands",
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
/// `|| true`, a stderr-to-`/dev/null` redirect, or a bare `;` joining two
/// shell commands (Make's exit status for a recipe line is its *last*
/// command's); and `$(eval` anywhere.
///
/// Returns every violation found; an empty result means the text is clean.
/// An unreadable file — including an `include` target that cannot be
/// resolved — is itself a violation rather than a silent skip: this check
/// fails closed on anything it cannot read, exactly as it fails closed on
/// anything it can read and does not like.
pub fn scan_makefile(path: &Path) -> Vec<Violation> {
    let mut visited = BTreeSet::new();
    let mut out = Vec::new();
    let exempt = exempt_missing_include_path(path);
    scan_file(path, true, &exempt, &mut visited, &mut out);
    out
}

/// The one path a missing soft-include is allowed to name without being a
/// violation (see [`scan_file`]): `ci_guard ci`'s own generated
/// `target/ci-gates/.gate-tokens.mk`, resolved against the directory the
/// *top-level* Makefile passed to [`scan_makefile`] lives in. Computed once,
/// not per include level, so a nested include cannot use a relative-path
/// trick (`../`) to make some other, differently-located missing target
/// spuriously match — every candidate is compared, after lexical
/// normalization, against this one fixed value.
fn exempt_missing_include_path(top_level_makefile: &Path) -> PathBuf {
    let base_dir = top_level_makefile
        .parent()
        .unwrap_or_else(|| Path::new("."));
    normalize_lexically(&base_dir.join(GATES_DIR).join(GATE_TOKENS_MK))
}

/// Resolve `.`/`..` components without touching the filesystem (the path
/// being checked may not exist — that is the whole point of the check this
/// supports). Not symlink-aware; matches this module's existing posture of
/// textual, not exhaustively adversarial, analysis (see
/// `has_bare_command_separator`'s doc comment for the same trade-off stated
/// explicitly elsewhere in this file).
fn normalize_lexically(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                if !out.pop() {
                    out.push("..");
                }
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// Scan `path`. `required` is `false` only for a target reached through a
/// `-include`/`sinclude` directive: Make itself silently continues when that
/// specific file does not exist (that is the documented difference from a
/// plain `include`, which stops Make with an error). Even then, a missing
/// target is treated as harmless-absent only when it is also `exempt_path`
/// (see [`exempt_missing_include_path`]) — *every other* missing soft
/// include is still flagged.
///
/// That narrowing is the fix for a regression an independent review of
/// NFR-004-AC-9 found and reproduced against the compiled binary (TL-202): a
/// Makefile carrying `-include generated.mk` where `generated.mk` does not
/// exist yet, alongside a Make *rule* to build `generated.mk` containing
/// `.IGNORE:` (or any other execution-control surface), passed this scan
/// cleanly before this narrowing — the file was simply absent at scan time.
/// But GNU Make's own documented behavior is to remake an included makefile
/// that has a rule for it and *restart itself* with the freshly built
/// version once remaking finishes, all before running any `ci` prerequisite
/// recipe — so the planted `.IGNORE:` would have taken effect regardless of
/// what this one-shot, before-Make-ever-runs scan saw. Exempting *only* the
/// one path this repository's own Makefile actually needs to `-include` —
/// `ci_guard ci`'s own generated, code-controlled `.gate-tokens.mk`, which
/// carries no rule of its own and is never itself Make-buildable — removes
/// the opening without reopening it for an arbitrary future soft-include a
/// Makefile edit might add. A file that exists but cannot be read for any
/// other reason (permissions, a directory in its place, …) is still flagged
/// regardless of `required` or path: fail closed on anything that is not the
/// exact, narrow "Make would have silently skipped this too, and nothing can
/// make it exist behind this scan's back" case.
fn scan_file(
    path: &Path,
    required: bool,
    exempt_path: &Path,
    visited: &mut BTreeSet<PathBuf>,
    out: &mut Vec<Violation>,
) {
    let canonical = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    if !visited.insert(canonical) {
        return; // already scanned on this walk; avoid an include cycle
    }
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(err)
            if !required
                && err.kind() == std::io::ErrorKind::NotFound
                && normalize_lexically(path) == exempt_path =>
        {
            return; // absent optional include of exactly the one exempt path
        }
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
            scan_directive_line(path, lineno, trimmed, base_dir, exempt_path, visited, out);
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
    if has_bare_command_separator(body) {
        push(
            ViolationKind::RecipeChainsCommands,
            "a bare `;` joins shell commands on one recipe line, so Make's \
             exit status for the line is the last command's, not an earlier \
             check's — e.g. `false; ci_guard record gate` reports success \
             because `ci_guard record` always exits 0"
                .to_string(),
        );
    }
}

/// True if `body` joins two shell commands with a bare `;` (a plain command
/// separator, not part of a `for`/`while`/`case` control-flow keyword or a
/// `;;` case-statement terminator). Make runs each recipe line as one shell
/// invocation whose exit status is the *last* command's; `cmd1; cmd2`
/// silently discards `cmd1`'s failure the same way `|| true` does, and is
/// how a recipe combining a check with `ci_guard record <gate>` (which
/// always exits 0) can report success despite the check failing — the
/// static scan must see it precisely because reconciliation cannot: a
/// record written that way is genuinely present and genuinely from this run.
///
/// Textual, not shell-grammar-aware: a `;` inside a quoted string is still
/// flagged, the same accepted false-positive-over-false-negative trade-off
/// as this module's other recipe-content checks.
fn has_bare_command_separator(body: &str) -> bool {
    const CONTROL_WORDS: [&str; 7] = ["do", "done", "then", "else", "elif", "fi", "esac"];
    let mut chars = body.char_indices();
    while let Some((idx, ch)) = chars.next() {
        if ch != ';' {
            continue;
        }
        if body[idx + 1..].starts_with(';') {
            chars.next(); // `;;` case-statement terminator, not a separator
            continue;
        }
        let rest = body[idx + 1..].trim_start();
        let is_control_word = CONTROL_WORDS.iter().any(|kw| {
            rest.strip_prefix(kw)
                .is_some_and(|after| !after.starts_with(|c: char| c.is_alphanumeric() || c == '_'))
        });
        if !is_control_word {
            return true;
        }
    }
    false
}

fn scan_directive_line(
    path: &Path,
    lineno: usize,
    trimmed: &str,
    base_dir: &Path,
    exempt_path: &Path,
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
    if let Some((target, required)) = include_target(trimmed) {
        let included = base_dir.join(target);
        scan_file(&included, required, exempt_path, visited, out);
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

/// The `include` target named on `line`, and whether Make treats a missing
/// target as an error (`include`, `required = true`) or silently continues
/// (`-include`/`sinclude`, `required = false`).
fn include_target(line: &str) -> Option<(&str, bool)> {
    for prefix in ["-include ", "sinclude "] {
        if let Some(rest) = line.strip_prefix(prefix) {
            return Some((rest.trim(), false));
        }
    }
    if let Some(rest) = line.strip_prefix("include ") {
        return Some((rest.trim(), true));
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

/// `true` if `gate` is safe to use as a bare filename component: every
/// declared `ci` prerequisite name in this repository's Makefile is
/// lowercase ASCII letters, digits, and `-` (e.g. `fmt-check`,
/// `check-corpus`), so that is the admitted alphabet. Rejects anything that
/// could escape the completion-record directory (`/`, `..`, a leading `.`)
/// along with anything simply outside the expected shape.
fn is_valid_gate_name(gate: &str) -> bool {
    !gate.is_empty()
        && gate
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

/// Write `gate`'s completion record into `dir`, stamped with `run_id`.
///
/// `gate` is a CLI argument in practice (`ci_guard record GATE`, called from
/// a Makefile recipe with a literal gate name); not reachable with an
/// attacker-controlled value today, since every caller is a fixed literal in
/// this repository's own trusted Makefile. Validated anyway, matching this
/// module's fail-closed-on-untrusted-shape default: an invalid `gate` is
/// refused with an error rather than silently building a path that could
/// escape `dir` (NFR-004's own threat model is exactly "a recipe edit
/// reopens something this control was supposed to close").
pub fn write_record(dir: &Path, gate: &str, run_id: &str) -> std::io::Result<()> {
    if !is_valid_gate_name(gate) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("refusing to write a completion record for invalid gate name {gate:?}"),
        ));
    }
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

/// A per-gate, per-run secret minted just before `make ci` runs and
/// delivered into only that gate's own recipe environment (NFR-004-AC-9,
/// TL-202). Keyed by gate name.
pub type GateTokens = BTreeMap<String, String>;

/// Where `ci_guard ci` keeps completion records and the generated per-gate
/// token files, relative to the directory the Makefile it drives lives in.
/// The single definition both `src/bin/ci_guard.rs` and [`scan_makefile`]'s
/// missing-soft-include exemption build on — see that exemption's own doc
/// comment for why the exemption is scoped to exactly this location rather
/// than to soft-includes generally (TL-202 review finding, post-AC-9).
pub const GATES_DIR: &str = "target/ci-gates";
const GATE_TOKENS_JSON: &str = ".gate-tokens.json";
const GATE_TOKENS_MK: &str = ".gate-tokens.mk";
/// The environment variable name a gate recipe's own `ci_guard record` call
/// reads its per-gate token from. Delivered by the generated Make fragment's
/// target-specific `export`, never by the top-level `make` invocation's own
/// environment — unlike [`GateRecord::run_id`]'s source, `CI_GUARD_RUN_ID`,
/// which every recipe and every subprocess it spawns inherits alike.
pub const GATE_TOKEN_VAR: &str = "CI_GUARD_GATE_TOKEN";

/// 32 bytes read from `/dev/urandom`, or `None` if that device cannot be
/// opened or a short read occurs (a non-Unix host, an unusually locked-down
/// sandbox, …). [`mint_gate_tokens`] still produces a usable, run-unique
/// token without it — see that function's documentation for what guarantee
/// is lost when this returns `None`.
fn urandom_bytes() -> Option<[u8; 32]> {
    let mut file = fs::File::open("/dev/urandom").ok()?;
    let mut buf = [0u8; 32];
    file.read_exact(&mut buf).ok()?;
    Some(buf)
}

/// Mint one fresh token per `gate` in `declared`, keyed to `run_id`.
///
/// Each token folds in `/dev/urandom` output (when available), wall-clock
/// time, this process's pid, the gate name, and a per-call counter, so that
/// no two tokens collide even if two gates are minted within the same
/// nanosecond or `/dev/urandom` is unavailable and the run falls back to the
/// other, lower-entropy inputs alone. Deliberately **not** a deterministic
/// function of `run_id` and `gate` alone: `run_id` is already visible to
/// every subprocess `make ci` spawns (that visibility is exactly what
/// TL-202 reports), so a token any such subprocess could recompute from
/// `run_id` and a gate name it already knows would bind nothing.
pub fn mint_gate_tokens(declared: &BTreeSet<String>, run_id: &str) -> GateTokens {
    declared
        .iter()
        .enumerate()
        .map(|(i, gate)| (gate.clone(), mint_one_token(run_id, gate, i as u64)))
        .collect()
}

fn mint_one_token(run_id: &str, gate: &str, counter: u64) -> String {
    let mut hasher = Sha256::new();
    hasher.update(run_id.as_bytes());
    hasher.update(b"\0");
    hasher.update(gate.as_bytes());
    hasher.update(b"\0");
    hasher.update(counter.to_le_bytes());
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default();
    hasher.update(nanos.to_le_bytes());
    hasher.update(std::process::id().to_le_bytes());
    if let Some(random) = urandom_bytes() {
        hasher.update(random);
    }
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// Write `tokens` two ways into `dir`: a JSON map [`read_gate_tokens`] reads
/// back to validate a `ci_guard record` call, and a generated Make fragment
/// (`.gate-tokens.mk`) the real Makefile `-include`s, which uses Make's own
/// target-specific-variable scoping to place exactly one gate's token into
/// exactly that gate's own recipe environment via `export` — never into a
/// sibling gate's recipe environment, and never into the top-level `make`
/// process's own environment the way `CI_GUARD_RUN_ID` is.
///
/// Both files are generated fresh by this function on every run from an
/// already-validated gate-name set (every declared `ci` prerequisite, parsed
/// from Makefile text `scan_makefile` has already cleared); neither is
/// human-edited, so unlike the Makefile itself, the fragment this writes is
/// not re-scanned for execution-control surfaces after being written.
/// Refuses (without writing anything) if any gate name fails
/// `is_valid_gate_name` — defense in depth matching [`write_record`]'s own
/// default, even though every caller passes a set already drawn from parsed
/// Makefile text.
pub fn write_gate_tokens(dir: &Path, tokens: &GateTokens) -> std::io::Result<()> {
    for gate in tokens.keys() {
        if !is_valid_gate_name(gate) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("refusing to write a gate token for invalid gate name {gate:?}"),
            ));
        }
    }
    fs::create_dir_all(dir)?;
    let json = serde_json::to_vec_pretty(tokens)
        .expect("GateTokens serialization is infallible for these field types");
    fs::write(dir.join(GATE_TOKENS_JSON), json)?;

    let mut mk = String::from(
        "# Generated by `ci_guard ci` (NFR-004-AC-9, TL-202). Do not edit or commit:\n\
         # rewritten fresh before every guarded run and absent otherwise.\n",
    );
    for (gate, token) in tokens {
        mk.push_str(gate);
        mk.push_str(": export ");
        mk.push_str(GATE_TOKEN_VAR);
        mk.push_str(" := ");
        mk.push_str(token);
        mk.push('\n');
    }
    fs::write(dir.join(GATE_TOKENS_MK), mk)
}

/// Read the token map [`write_gate_tokens`] wrote into `dir`. A missing
/// directory, a missing file, or a file that fails to parse contributes an
/// empty map — the same "no token available" input to [`authorized_gate`]
/// as an absent record is to [`reconcile`], never an authorization.
pub fn read_gate_tokens(dir: &Path) -> GateTokens {
    let Ok(bytes) = fs::read(dir.join(GATE_TOKENS_JSON)) else {
        return GateTokens::new();
    };
    serde_json::from_slice(&bytes).unwrap_or_default()
}

/// `true` iff `gate` was minted a token in `tokens` and `presented` is
/// `Some` of exactly that value.
///
/// This binds a `ci_guard record` call to the recipe environment Make itself
/// scoped to that gate (NFR-004-AC-9): a subprocess spawned during a
/// *different* declared gate's own recipe — the concrete risk TL-202 names,
/// e.g. a compromised or buggy dependency invoked by `cargo test`, `cargo
/// clippy`, a Python script, or an external `quire`/`quoin` binary — does
/// not have this gate's token in its own inherited environment the way it
/// already has `CI_GUARD_RUN_ID`, and so cannot satisfy this check for any
/// gate other than its own without separately locating and reading this
/// module's token store off disk. That residual — a subprocess with general
/// filesystem access during the run that goes looking for the token store
/// rather than merely inheriting an environment variable — is disclosed in
/// NFR-004's Scope rather than claimed closed: Make gives every recipe in a
/// single `make ci` invocation the same user, the same filesystem, and no
/// sandboxing between them, so no protocol built only from environment
/// variables and files can withhold a value from a sufficiently deliberate
/// reader in that same trust domain. Closing that residual would require
/// running each gate's recipe as a genuinely separate OS principal, which is
/// a materially larger change than this control.
pub fn authorized_gate(tokens: &GateTokens, gate: &str, presented: Option<&str>) -> bool {
    match (tokens.get(gate), presented) {
        (Some(expected), Some(presented)) => expected == presented,
        _ => false,
    }
}

/// Delete `dir`'s gate-token files, best-effort. Called after reconciliation
/// so a completed run's tokens do not sit on disk longer than the run that
/// minted them — not itself a security boundary ([`reset_gates_dir`] already
/// wipes any leftover files at the *start* of the next run before minting
/// fresh ones — an attacker cannot make a prior run's token accepted by a
/// later run's [`authorized_gate`] check, which is keyed to that later run's
/// own freshly minted map), just hygiene.
pub fn cleanup_gate_tokens(dir: &Path) {
    let _ = fs::remove_file(dir.join(GATE_TOKENS_JSON));
    let _ = fs::remove_file(dir.join(GATE_TOKENS_MK));
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
    // Independent second-pass review (SR-079/FND-001): a recipe line joining
    // the check and the record call with a bare `;` is invisible to every
    // other check here (no .IGNORE, no `-` prefix, no `|| true`, no stderr
    // redirect, no $(eval)) yet defeats reconciliation, because Make's exit
    // status for the line is `ci_guard record`'s (always 0), not the
    // check's. This is the reviewer's own reproduction shape.
    #[test]
    fn scan_detects_semicolon_chained_check_and_record() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp(
            dir.path(),
            "Makefile",
            "ci: gate-a\ngate-a:\n\tfalse; \"/path/to/ci_guard\" record gate-a\n",
        );
        assert!(scan_makefile(&path)
            .iter()
            .any(|v| v.kind == ViolationKind::RecipeChainsCommands));
    }

    // Trace: TC-057, NFR-004-AC-1
    #[test]
    fn scan_detects_simple_semicolon_chain() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp(dir.path(), "Makefile", "ci:\n\ttrue; false\n");
        assert!(scan_makefile(&path)
            .iter()
            .any(|v| v.kind == ViolationKind::RecipeChainsCommands));
    }

    // Trace: TC-057, NFR-004-AC-1
    // A `for`/`do`/`done` loop's structural semicolons are not a command
    // separator hiding a failure and must not be flagged — over-flagging
    // ordinary shell control flow would make this check impractical to
    // leave on for any recipe using a loop.
    #[test]
    fn scan_allows_for_loop_control_flow_semicolons() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp(
            dir.path(),
            "Makefile",
            "ci:\n\tfor f in a b c; do echo $$f; done\n",
        );
        assert!(scan_makefile(&path).is_empty());
    }

    // Trace: TC-057, NFR-004-AC-1
    // `;;` terminates a `case` branch; it is not two bare command separators.
    #[test]
    fn scan_allows_case_statement_terminators() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp(
            dir.path(),
            "Makefile",
            "ci:\n\tcase $$x in a) true ;; b) true ;; esac\n",
        );
        assert!(scan_makefile(&path).is_empty());
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

    // Trace: TC-064, NFR-004-AC-9
    // A missing `-include`/`sinclude` of *exactly* `ci_guard ci`'s own
    // generated `target/ci-gates/.gate-tokens.mk` is not a violation: Make
    // itself silently continues past a missing soft-include, unlike a plain
    // `include`, and this is the one path the real Makefile permanently
    // `-include`s that is legitimately absent outside a `ci_guard ci` run (a
    // bare `make ci`, `make lint`, or a fresh checkout). Both spellings
    // (`-include`/`sinclude`) are exempt for this exact path.
    #[test]
    fn scan_does_not_flag_a_missing_soft_include_of_the_exempt_gate_tokens_path() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp(
            dir.path(),
            "Makefile",
            "-include target/ci-gates/.gate-tokens.mk\nci:\n\tfalse\n",
        );
        assert!(scan_makefile(&path).is_empty());

        let path2 = write_temp(
            dir.path(),
            "Makefile2",
            "sinclude target/ci-gates/.gate-tokens.mk\nci:\n\tfalse\n",
        );
        assert!(scan_makefile(&path2).is_empty());
    }

    // Trace: TC-064, NFR-004-AC-9
    // Regression (independent review, post-AC-9): the missing-soft-include
    // exemption must be scoped to exactly the one generated path above, not
    // to soft-includes generally. Before this was scoped, a Makefile could
    // carry `-include generated.mk` (missing at scan time, so unflagged)
    // alongside a Make *rule* to build `generated.mk` containing `.IGNORE:`
    // — GNU Make remakes an included file it has a rule for and restarts
    // itself with the freshly built version before running any `ci`
    // prerequisite, planting the directive after this scan already passed.
    // A missing soft-include of any path *other* than the one exempt path
    // must still be flagged, closing that reopening.
    #[test]
    fn scan_still_flags_a_missing_soft_include_of_any_other_path() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp(
            dir.path(),
            "Makefile",
            "-include generated.mk\nci:\n\tfalse\n",
        );
        assert!(scan_makefile(&path)
            .iter()
            .any(|v| v.kind == ViolationKind::Unreadable));

        // Even a path that merely *ends with* the exempt path's components,
        // rooted somewhere else, must not be forgiven — only an exact match
        // (after lexical normalization) against the one path resolved from
        // this Makefile's own directory is exempt.
        let path2 = write_temp(
            dir.path(),
            "Makefile2",
            "-include elsewhere/target/ci-gates/.gate-tokens.mk\nci:\n\tfalse\n",
        );
        assert!(scan_makefile(&path2)
            .iter()
            .any(|v| v.kind == ViolationKind::Unreadable));
    }

    // Trace: TC-064, NFR-004-AC-9
    // The reviewer's exact reproduction, at the `scan_makefile` unit level:
    // a non-exempt soft-include target that a Make rule could build is still
    // flagged as unreadable while genuinely missing, regardless of whether a
    // rule to build it exists elsewhere in the same file — the scan runs
    // once, before Make (and hence before any such rule could ever run), so
    // it can only ever see "missing" or "present", never "buildable".
    // Confirms the fix does not accidentally key off "does a rule exist" (a
    // check this static, single-pass scanner cannot make reliably) but
    // simply refuses every non-exempt missing target outright.
    #[test]
    fn scan_flags_a_missing_soft_include_even_when_a_rule_could_build_it() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp(
            dir.path(),
            "Makefile",
            "-include generated.mk\nci: gate-a\ngate-a:\n\tfalse\n\t\"guard\" record gate-a\n\
             generated.mk:\n\techo '.IGNORE:' > generated.mk\n",
        );
        assert!(scan_makefile(&path)
            .iter()
            .any(|v| v.kind == ViolationKind::Unreadable));
    }

    // Trace: TC-064, NFR-004-AC-9
    // A `-include`/`sinclude` target that *does* exist is scanned exactly
    // like a hard `include` — only missingness is forgiven, not content.
    #[test]
    fn scan_still_scans_a_present_soft_include() {
        let dir = tempfile::tempdir().unwrap();
        write_temp(dir.path(), "extra.mk", ".IGNORE:\n");
        let path = write_temp(dir.path(), "Makefile", "-include extra.mk\nci:\n\tfalse\n");
        assert!(scan_makefile(&path)
            .iter()
            .any(|v| v.kind == ViolationKind::IgnoreDirective && v.file.ends_with("extra.mk")));
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

    // Trace: TC-059, NFR-004-AC-3
    // SR-079/FND-003: `gate` reaches the filesystem path unvalidated; a
    // traversal-shaped name must be refused rather than escaping `dir`.
    #[test]
    fn write_record_rejects_path_traversal_gate_names() {
        let dir = tempfile::tempdir().unwrap();
        for gate in ["../escape", "a/b", "/etc/passwd", "..", "", "Fmt-Check"] {
            let result = write_record(dir.path(), gate, "run-1");
            assert!(result.is_err(), "expected {gate:?} to be refused");
        }
        // Nothing escaped `dir`.
        assert!(read_records(dir.path()).is_empty());
        assert!(!dir.path().parent().unwrap().join("escape").exists());
    }

    // Trace: TC-059, NFR-004-AC-3
    #[test]
    fn write_record_accepts_every_real_gate_name() {
        let dir = tempfile::tempdir().unwrap();
        for gate in [
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
        ] {
            write_record(dir.path(), gate, "run-1").unwrap();
        }
        assert_eq!(read_records(dir.path()).len(), 13);
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

    // Trace: TC-064, NFR-004-AC-9
    #[test]
    fn mint_gate_tokens_covers_every_declared_gate_with_distinct_nonempty_tokens() {
        let declared: BTreeSet<String> = ["gate-a", "gate-b", "gate-c"]
            .into_iter()
            .map(String::from)
            .collect();
        let tokens = mint_gate_tokens(&declared, "run-1");
        assert_eq!(tokens.len(), 3);
        for gate in &declared {
            assert!(!tokens[gate].is_empty());
        }
        // No two gates share a token, even minted in the same call.
        let values: BTreeSet<&String> = tokens.values().collect();
        assert_eq!(values.len(), 3);
    }

    // Trace: TC-064, NFR-004-AC-9
    #[test]
    fn mint_gate_tokens_differs_across_runs_for_the_same_gate() {
        let declared: BTreeSet<String> = ["gate-a"].into_iter().map(String::from).collect();
        let first = mint_gate_tokens(&declared, "run-1");
        let second = mint_gate_tokens(&declared, "run-2");
        assert_ne!(first["gate-a"], second["gate-a"]);
    }

    // Trace: TC-064, NFR-004-AC-9
    #[test]
    fn write_then_read_gate_tokens_round_trips() {
        let dir = tempfile::tempdir().unwrap();
        let declared: BTreeSet<String> = ["fmt-check", "lint"]
            .into_iter()
            .map(String::from)
            .collect();
        let tokens = mint_gate_tokens(&declared, "run-1");
        write_gate_tokens(dir.path(), &tokens).unwrap();

        let read_back = read_gate_tokens(dir.path());
        assert_eq!(read_back, tokens);

        // The generated Make fragment scopes each token to its own target
        // via a target-specific `export` assignment, not a plain (globally
        // inherited) variable.
        let mk = fs::read_to_string(dir.path().join(".gate-tokens.mk")).unwrap();
        for (gate, token) in &tokens {
            assert!(mk.contains(&format!("{gate}: export {GATE_TOKEN_VAR} := {token}")));
        }
    }

    // Trace: TC-064, NFR-004-AC-9
    #[test]
    fn read_gate_tokens_is_empty_for_a_missing_store() {
        let dir = tempfile::tempdir().unwrap();
        assert!(read_gate_tokens(dir.path().join("absent").as_path()).is_empty());
    }

    // Trace: TC-064, NFR-004-AC-9
    // The reproduction TL-202 reports: a subprocess of gate-a's own recipe
    // knows CI_GUARD_RUN_ID and gate-b's *name* (both are already public —
    // the name is literally in the Makefile) but does not have gate-b's
    // token, because that token was only ever placed into gate-b's own
    // recipe environment. Confirms the fix in the reverse direction too: the
    // legitimate call, presenting its own gate's own token, is authorized.
    #[test]
    fn authorized_gate_rejects_a_different_gates_token_and_accepts_its_own() {
        let declared: BTreeSet<String> =
            ["gate-a", "gate-b"].into_iter().map(String::from).collect();
        let tokens = mint_gate_tokens(&declared, "run-1");

        // gate-a's subprocess presents its own token while claiming gate-b.
        assert!(!authorized_gate(
            &tokens,
            "gate-b",
            Some(tokens["gate-a"].as_str())
        ));
        // No token presented at all (e.g. CI_GUARD_GATE_TOKEN unset).
        assert!(!authorized_gate(&tokens, "gate-b", None));
        // An outright guessed/empty token.
        assert!(!authorized_gate(&tokens, "gate-b", Some("")));
        // gate-b's own recipe, presenting gate-b's own token: authorized.
        assert!(authorized_gate(
            &tokens,
            "gate-b",
            Some(tokens["gate-b"].as_str())
        ));
    }

    // Trace: TC-064, NFR-004-AC-9
    #[test]
    fn authorized_gate_rejects_an_undeclared_gate_name() {
        let declared: BTreeSet<String> = ["gate-a"].into_iter().map(String::from).collect();
        let tokens = mint_gate_tokens(&declared, "run-1");
        assert!(!authorized_gate(&tokens, "gate-z", Some("anything")));
    }

    // Trace: TC-064, NFR-004-AC-9
    #[test]
    fn write_gate_tokens_rejects_invalid_gate_names() {
        let dir = tempfile::tempdir().unwrap();
        let mut tokens = GateTokens::new();
        tokens.insert("../escape".to_string(), "token".to_string());
        assert!(write_gate_tokens(dir.path(), &tokens).is_err());
        assert!(!dir.path().join(GATE_TOKENS_JSON).exists());
    }

    // Trace: TC-064, NFR-004-AC-9
    #[test]
    fn cleanup_gate_tokens_removes_both_files() {
        let dir = tempfile::tempdir().unwrap();
        let declared: BTreeSet<String> = ["gate-a"].into_iter().map(String::from).collect();
        let tokens = mint_gate_tokens(&declared, "run-1");
        write_gate_tokens(dir.path(), &tokens).unwrap();
        cleanup_gate_tokens(dir.path());
        assert!(!dir.path().join(GATE_TOKENS_JSON).exists());
        assert!(!dir.path().join(GATE_TOKENS_MK).exists());
    }
}

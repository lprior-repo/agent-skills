//! Proof-planner validator library.
//!
//! Pure core: load → parse → check → emit. IO shell: main.rs.
//! No unwrap, no panic, no expect in this module. All fallible ops return
//! `Result<T, ValidatorError>` per holzman-rust doctrine.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use regex::Regex;
use serde::Serialize;
use serde_json::Value;
use thiserror::Error;

pub const ALLOWED_VERIFIERS: &[&str] = &[
    "verus",
    "kani",
    "flux-rs",
    "loom",
    "miri",
    "cargo-fuzz",
    "proptest",
];

pub const ALLOWED_APPLICABILITY: &[&str] = &["required", "not_applicable", "blocked_tooling"];
pub const ALLOWED_LIMITATION_KIND: &[&str] = &[
    "surface_absent",
    "risk_out_of_scope",
    "superseded_by_other_lane_with_evidence",
    "external_dependency_unavoidable",
];
pub const ALLOWED_MODES: &[&str] = &["verify-proof", "verify-smoke"];

pub const OBLIGATION_REQUIRED_FIELDS: &[&str] = &[
    "schema_version", "id", "requirement_id", "contract_clause",
    "domain_claim", "risk", "risk_tags", "verifier", "artifact",
    "target", "command", "workdir", "expected_evidence",
    "assumptions", "model_bounds", "tool_metadata",
    "trusted_base_refs", "required", "behavior_affecting",
    "mode", "owner_state", "rerun_from", "status",
];

pub const LANE_REQUIRED_FIELDS: &[&str] = &[
    "schema_version", "id", "requirement_id", "contract_clause",
    "proof_seed_id", "verifier", "risk_tags", "applicability",
    "decision_reason", "required_obligation_ids",
    "non_applicability_evidence_refs", "limitation_kind",
    "owner_state", "status",
];

pub const WAIVER_REQUIRED_FIELDS: &[&str] = &[
    "schema_version", "id", "requirement_id", "contract_clause",
    "reason", "behavior_affecting", "boundary_proof",
    "compensating_evidence", "owner", "expiry", "review_status",
];

pub const OBLIGATION_FORBIDDEN_ALIASES: &[&str] = &["layer", "checker", "claim"];

pub const TOOL_MARKERS: &[&str] = &[
    "VERIFICATION:- SUCCESSFUL",
    "VERIFICATION:-",
    "Model checking completed",
    "test result: ok",
    "check passed",
    "should_fail",
    "rustc exited",
    "Verification SUCCEEDED",
    "Verification FAILED",
    "no UB diagnostic",
    "no diagnostic",
    "Verification successful",
    "verified safe",
];

/// Default Rust-behavior risk profile. Each risk class names the minimum
/// verifier lanes that must appear as `applicability: required` for a
/// behavior-affecting obligation. Planner rows that omit a default-profile
/// verifier without a waiver trigger E_LANE_DECISION_MISSING, unless the
/// verifier is covered by an accepted `not_applicable` lane decision: a
/// `verifier-lane-decision/v1` row with concrete SHA-256 evidence refs, a
/// typed `limitation_kind`, a concrete reason, and a matching
/// `verifier-lane-review/v1` row with `reviewer_disposition: accepted` from
/// an independent reviewer invocation (see `accepted_na_lanes`).
pub const DEFAULT_RISK_PROFILE: &[(&str, &[&str])] = &[
    ("arithmetic_overflow", &["verus", "kani", "proptest"]),
    ("index_safety", &["verus", "kani", "proptest"]),
    ("panic_freedom", &["verus", "kani", "proptest"]),
    ("rejection", &["kani", "proptest"]),
    ("bounded_transition", &["kani", "verus"]),
    ("equality", &["proptest", "verus"]),
    ("ordering", &["proptest", "verus"]),
    ("field_sensitivity", &["proptest", "kani"]),
    ("illegal_state", &["flux-rs", "verus"]),
    ("refinement", &["flux-rs", "verus"]),
    ("concurrency_interleaving", &["loom", "proptest"]),
    ("cancellation_safety", &["loom", "proptest"]),
    ("shutdown_drain", &["loom", "proptest"]),
    ("temporal_liveness", &["loom", "proptest"]),
    ("temporal_safety", &["verus", "loom"]),
    ("ub_safety", &["miri", "verus", "proptest"]),
    ("hostile_input", &["cargo-fuzz", "kani", "proptest"]),
    ("parse_canonicalization", &["cargo-fuzz", "verus", "kani"]),
];

pub fn risk_profile_required(risk: &str) -> Option<&'static [&'static str]> {
    DEFAULT_RISK_PROFILE
        .iter()
        .find(|(name, _)| *name == risk)
        .map(|(_, lanes)| *lanes)
}

#[derive(Debug, Error)]
pub enum ValidatorError {
    #[error("path is not a directory: {0}")]
    NotADirectory(PathBuf),
    #[error("I/O error on {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct Finding {
    pub schema_version: &'static str,
    pub reviewer_skill: &'static str,
    pub review_state: &'static str,
    pub finding_code: String,
    pub severity: Severity,
    pub artifact: String,
    pub owner_state: u8,
    pub message: String,
    pub required_fix: String,
    pub disposition: &'static str,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Blocker,
    Major,
}

#[derive(Debug, Default, Serialize)]
pub struct Summary {
    pub plan_dir: String,
    pub obligations: usize,
    pub lane_decisions: usize,
    pub waivers: usize,
    pub lane_reviews: usize,
    pub findings_total: usize,
    pub findings_blocker: usize,
    pub findings_major: usize,
    pub status: &'static str,
}

#[derive(Debug, Default)]
pub struct Plan {
    pub obligations: Vec<Value>,
    pub lane_decisions: Vec<Value>,
    pub waivers: Vec<Value>,
    pub lane_reviews: Vec<Value>,
}

pub fn load_plan(plan_dir: &Path) -> Result<Plan, ValidatorError> {
    if !plan_dir.is_dir() {
        return Err(ValidatorError::NotADirectory(plan_dir.to_path_buf()));
    }
    let obligations = load_jsonl(&plan_dir.join("proof-obligations.planned.jsonl"))?;
    let lane_decisions = load_jsonl(&plan_dir.join("verifier-lane-decisions.jsonl"))?;
    let waivers = load_jsonl(&plan_dir.join("waiver-candidates.jsonl"))?;
    let lane_reviews = load_jsonl(&plan_dir.join("verifier-lane-reviews.jsonl"))?;
    Ok(Plan {
        obligations,
        lane_decisions,
        waivers,
        lane_reviews,
    })
}

pub fn load_jsonl(path: &Path) -> Result<Vec<Value>, ValidatorError> {
    let text = match fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(source) => return Err(ValidatorError::Io { path: path.to_path_buf(), source }),
    };
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let value: Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(_) => {
                let mut err = serde_json::Map::new();
                err.insert("_parse_error".into(), Value::String(format!("line {}", i + 1)));
                err.insert("_line".into(), Value::Number((i as u64 + 1).into()));
                out.push(Value::Object(err));
                continue;
            }
        };
        out.push(value);
    }
    Ok(out)
}

pub fn row_id<'a>(row: &'a Value, default: &'a str) -> &'a str {
    row.get("id").and_then(Value::as_str).unwrap_or(default)
}

pub fn row_str<'a>(row: &'a Value, key: &str) -> Option<&'a str> {
    row.get(key).and_then(Value::as_str)
}

pub fn row_array<'a>(row: &'a Value, key: &str) -> Option<&'a Vec<Value>> {
    row.get(key).and_then(Value::as_array)
}

pub fn row_object<'a>(row: &'a Value, key: &str) -> Option<&'a serde_json::Map<String, Value>> {
    row.get(key).and_then(Value::as_object)
}

pub fn row_bool(row: &Value, key: &str) -> Option<bool> {
    row.get(key).and_then(Value::as_bool)
}

pub fn emit(
    findings: &mut Vec<Finding>,
    code: &str,
    severity: Severity,
    artifact: &str,
    msg: impl Into<String>,
    fix: impl Into<String>,
) {
    findings.push(Finding {
        schema_version: "finding/v1",
        reviewer_skill: "proof-planner/validate-plan",
        review_state: "self_audit",
        finding_code: code.to_string(),
        severity,
        artifact: artifact.to_string(),
        owner_state: 4,
        message: msg.into(),
        required_fix: fix.into(),
        disposition: "blocker",
    });
}

/// Check that an obligation's `target` names a production symbol, not just a file path.
pub fn check_target_shape(target: &str) -> bool {
    if target.is_empty() {
        return false;
    }
    if target.ends_with(".rs") && !target.contains("::") {
        return false;
    }
    target.contains("::")
}

/// Detect generic command patterns that fail the evidence-specificity bar.
pub fn check_command_specificity(cmd: &str) -> bool {
    if cmd.is_empty() {
        return false;
    }
    let placeholders = ["<", ">"];
    if placeholders.iter().any(|p| cmd.contains(p)) {
        return false;
    }
    if cmd.contains("...") {
        return false;
    }
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.len() <= 2 {
        return false;
    }
    if cmd.starts_with("cargo test") {
        let has_specific_target = cmd.contains("--test ")
            || cmd.contains("--bench ")
            || cmd.contains("--example ")
            || cmd.contains("--harness ")
            || cmd.contains("--doc ");
        return has_specific_target;
    }
    let generic_subcommands = ["build", "check"];
    if parts.len() <= 3 && generic_subcommands.contains(&parts[1]) {
        return false;
    }
    true
}

/// Verify that `expected_evidence` cites a concrete tool success marker.
pub fn check_tool_marker(ev: &str) -> bool {
    if ev.is_empty() {
        return false;
    }
    let lower = ev.to_lowercase();
    TOOL_MARKERS.iter().any(|m| lower.contains(&m.to_lowercase()))
}

/// Lazy regex compilation. Patterns are verified once and reused.
pub struct Patterns {
    pub verus_launder: [Regex; 3],
    pub kani_cover: Regex,
    pub kani_disabled: Regex,
    pub kani_hardcoded: Regex,
    pub loom_std_sync: Regex,
    pub loom_cfg_lo_present: Regex,
    pub miri_present: Regex,
    pub miri_mirflags_present: Regex,
    pub fuzz_run: Regex,
    pub proptest_run: Regex,
    pub kani_j_gt_one: Regex,
    pub kani_assert_or_contract: Regex,
    pub iso8601_prefix: Regex,
    pub weak_reason: Vec<Regex>,
}

impl Default for Patterns {
    fn default() -> Self {
        Self::new()
    }
}

impl Patterns {
    /// Compile every regex literal. Every pattern is a static literal in this
    /// function; compilation cannot fail at runtime unless the `regex` crate
    /// changes its syntax. The unit test `patterns_new_compiles_all_regexes`
    /// asserts the same patterns compile. We allow `expect_used` here
    /// because (a) the patterns are static, (b) the call is module-init only,
    /// and (c) a regex-syntax regression should crash the process, not be
    /// silently swallowed.
    #[allow(clippy::expect_used)]
    pub fn new() -> Self {
        Self {
            verus_launder: [
                Regex::new(r"(?i)external_body").expect("static regex"),
                Regex::new(r"assume\s*\(").expect("static regex"),
                Regex::new(r"\baxiom\b").expect("static regex"),
            ],
            kani_cover: Regex::new(r"\bcover!\s*\(").expect("static regex"),
            kani_disabled: Regex::new(
                r"--no-default-checks|--no-memory-safety-checks|--no-overflow-checks|--no-unwinding-checks|--prove-safety-only|--only-codegen|--no-codegen",
            )
            .expect("static regex"),
            kani_hardcoded: Regex::new(
                r#"let\s+\w+\s*=\s*vec!\s*\[|let\s+\w+\s*=\s*\[\s*0u?8|let\s+\w+\s*=\s*"""#,
            )
            .expect("static regex"),
            loom_std_sync: Regex::new(r"std::sync::(Mutex|RwLock|mpsc|Arc)").expect("static regex"),
            loom_cfg_lo_present: Regex::new(r"--cfg\s+loom").expect("static regex"),
            miri_present: Regex::new(r"cargo\s+\+nightly\s+miri\s+test").expect("static regex"),
            miri_mirflags_present: Regex::new(r"MIRIFLAGS=").expect("static regex"),
            fuzz_run: Regex::new(r"cargo\s+fuzz\s+run\s+\S+").expect("static regex"),
            proptest_run: Regex::new(r"cargo\s+test").expect("static regex"),
            kani_j_gt_one: Regex::new(r"cargo\s+kani\s+-j\s+[2-9]").expect("static regex"),
            kani_assert_or_contract: Regex::new(r"kani::assert|::requires|::ensures").expect("static regex"),
            iso8601_prefix: Regex::new(r"^\d{4}-\d{2}-\d{2}").expect("static regex"),
            weak_reason: vec![
                Regex::new(r"(?i)^not needed\.?$").expect("static regex"),
                Regex::new(r"(?i)^too hard\.?$").expect("static regex"),
                Regex::new(r"(?i)^not practical\.?$").expect("static regex"),
                Regex::new(r"(?i)^we'?ll add this later\.?$").expect("static regex"),
                Regex::new(r"(?i)^low risk\.?$").expect("static regex"),
                Regex::new(r"(?i)^trivial enough\.?$").expect("static regex"),
                Regex::new(r"(?i)^covered by (other|tests?)\.?$").expect("static regex"),
            ],
        }
    }
}

pub fn run_all_checks(plan: &Plan, patterns: &Patterns, findings: &mut Vec<Finding>) {
    check_obligations(&plan.obligations, patterns, findings);
    check_lane_decisions(&plan.lane_decisions, &plan.obligations, findings);
    check_waivers(&plan.waivers, findings);
    check_risk_profile_coverage(
        &plan.obligations,
        &plan.lane_decisions,
        &plan.lane_reviews,
        findings,
    );
}

fn check_obligations(obligations: &[Value], patterns: &Patterns, findings: &mut Vec<Finding>) {
    let mut seen_ids: BTreeSet<String> = BTreeSet::new();
    for row in obligations {
        if row.get("_parse_error").is_some() {
            emit(findings, "E_SCHEMA_MISSING_FIELD", Severity::Blocker,
                "proof-obligations.planned.jsonl",
                format!("JSON parse error: {}", row.get("_parse_error").and_then(Value::as_str).unwrap_or("?")),
                "Fix the JSON syntax; every row must be a valid object.",
            );
            continue;
        }
        check_one_obligation(row, patterns, &mut seen_ids, findings);
    }
}

fn check_one_obligation(
    row: &Value,
    patterns: &Patterns,
    seen_ids: &mut BTreeSet<String>,
    findings: &mut Vec<Finding>,
) {
    let id = row_id(row, "?");
    let artifact = format!("proof-obligations.planned.jsonl::{id}");

    let schema_version = row_str(row, "schema_version").unwrap_or("");
    if schema_version != "proof-obligation/v1" {
        emit(findings, "E_SCHEMA_VERSION_MISSING", Severity::Blocker, &artifact,
            format!("schema_version is {schema_version:?}; expected 'proof-obligation/v1'"),
            "Set schema_version to the literal 'proof-obligation/v1'.");
        return;
    }

    for alias in OBLIGATION_FORBIDDEN_ALIASES {
        if row.get(*alias).is_some() {
            emit(findings, "E_SCHEMA_ALIAS_FIELD", Severity::Blocker, &artifact,
                format!("forbidden alias field {alias:?}; use 'verifier' / 'domain_claim' instead"),
                format!("Remove the {alias:?} field."));
        }
    }

    for f in OBLIGATION_REQUIRED_FIELDS {
        match row.get(*f) {
            None => {
                emit(findings, "E_SCHEMA_MISSING_FIELD", Severity::Blocker, &artifact,
                    format!("missing required field {f:?}"),
                    format!("Add the {f:?} field with a non-empty value."));
            }
            Some(v) if v.is_null() => {
                emit(findings, "E_SCHEMA_MISSING_FIELD", Severity::Blocker, &artifact,
                    format!("required field {f:?} is null"),
                    format!("Set {f:?} to a non-empty value."));
            }
            _ => {}
        }
    }

    if let Some(rid) = row_str(row, "id") {
        if !seen_ids.insert(rid.to_string()) {
            emit(findings, "E_LANE_DECISION_DUPLICATE", Severity::Blocker, &artifact,
                format!("duplicate obligation id {rid:?}"),
                "Assign a unique PO-NNN id to each obligation.");
        }
    }

    let verifier = row_str(row, "verifier").unwrap_or("");
    if !ALLOWED_VERIFIERS.contains(&verifier) {
        emit(findings, "E_SCHEMA_MISSING_FIELD", Severity::Blocker, &artifact,
            format!("verifier {verifier:?} is not in the allowed set {ALLOWED_VERIFIERS:?}"),
            format!("Use one of {ALLOWED_VERIFIERS:?}."));
        return;
    }

    let mode = row_str(row, "mode").unwrap_or("");
    if !ALLOWED_MODES.contains(&mode) {
        emit(findings, "E_SCHEMA_MISSING_FIELD", Severity::Blocker, &artifact,
            format!("mode {mode:?} must be one of {ALLOWED_MODES:?}"),
            "Set mode to 'verify-proof' or 'verify-smoke'.");
    }

    let status = row_str(row, "status").unwrap_or("");
    if !matches!(status, "planned" | "blocked_tooling") {
        emit(findings, "E_SCHEMA_MISSING_FIELD", Severity::Blocker, &artifact,
            format!("status {status:?} is not allowed for planner rows"),
            "Planner rows are 'planned' or 'blocked_tooling'; PASS is verifier output only.");
    }

    let target = row_str(row, "target").unwrap_or("");
    if !check_target_shape(target) {
        emit(findings, "E_SOURCE_REF_SHAPE", Severity::Blocker, &artifact,
            format!("target {target:?} is not a production symbol"),
            "Use 'crate::path::symbol' or 'path/to/file.rs::symbol'.");
    }

    let cmd = row_str(row, "command").unwrap_or("");
    if !check_command_specificity(cmd) {
        emit(findings, "E_COMMAND_EVIDENCE_MISSING", Severity::Blocker, &artifact,
            format!("command {cmd:?} is generic; harness/target/test must be named"),
            "Name the exact harness, test, target, and feature flags.");
    }

    let ev = row_str(row, "expected_evidence").unwrap_or("");
    if !check_tool_marker(ev) {
        emit(findings, "E_PROOF_SMOKE_MISSING", Severity::Blocker, &artifact,
            format!("expected_evidence {ev:?} lacks a concrete tool marker"),
            "Cite the exact success line emitted by the verifier (e.g., 'VERIFICATION:- SUCCESSFUL').");
    }

    if matches!(verifier, "kani" | "loom" | "cargo-fuzz" | "proptest") {
        let mb = row_object(row, "model_bounds");
        let mb_empty = mb.map(|o| o.is_empty()).unwrap_or(true);
        if mb_empty {
            emit(findings, "E_PROOF_PLAN_MISSING_NONVACUITY", Severity::Blocker, &artifact,
                format!("model_bounds is empty for bounded verifier {verifier:?}"),
                "Populate model_bounds with the relevant bound fields per references/resource-governance.md.");
        }
    }

    let ctx = VerifierCtx { row, verifier, cmd, target, ev, artifact: &artifact, patterns };
    check_verifier_specific(&ctx, findings);

    let tm = row_object(row, "tool_metadata");
    let version_pin = tm.and_then(|t| t.get("version_pin")).and_then(Value::as_str).unwrap_or("");
    if version_pin.is_empty() {
        emit(findings, "E_PROOF_SMOKE_MISSING", Severity::Blocker, &artifact,
            "tool_metadata.version_pin is empty",
            "Pin the tool version (e.g., 'cargo-kani@0.65', 'verus@0.2024.10').");
    }

    let assumptions = row_array(row, "assumptions").map(|a| !a.is_empty()).unwrap_or(false);
    let tbrs = row_array(row, "trusted_base_refs").map(|a| !a.is_empty()).unwrap_or(false);
    if assumptions && !tbrs {
        emit(findings, "E_TRUST_LEDGER_INCOMPLETE", Severity::Blocker, &artifact,
            "obligations with non-empty assumptions must list trusted_base_refs",
            "Add at least one trusted_base_refs entry per assumption.");
    }

    check_artifact_source(row, verifier, patterns, &artifact, findings);
}

/// Per-verifier check context. Bundles the 8 fields that previously made
/// `check_verifier_specific` exceed clippy's 7-argument ceiling.
struct VerifierCtx<'a> {
    row: &'a Value,
    verifier: &'a str,
    cmd: &'a str,
    target: &'a str,
    ev: &'a str,
    artifact: &'a str,
    patterns: &'a Patterns,
}

fn check_verifier_specific(ctx: &VerifierCtx<'_>, findings: &mut Vec<Finding>) {
    let VerifierCtx { row, verifier, cmd, target, ev, artifact, patterns } = ctx;
    match *verifier {
        "verus" => {
            let cmd_target = format!("{cmd}\n{target}");
            for pat in &patterns.verus_launder {
                if pat.is_match(&cmd_target) {
                    emit(findings, "E_VERUS_DISCONNECTED_SPEC", Severity::Blocker, artifact,
                        format!("verus obligation references forbidden pattern in command/target: {}", pat.as_str()),
                        "Prove the real exec fn body; forbid external_body / assume / axiom in command and target.");
                }
            }
        }
        "kani" => {
            if patterns.kani_cover.is_match(cmd) && !patterns.kani_assert_or_contract.is_match(cmd) {
                emit(findings, "E_KANI_COVER_ONLY", Severity::Blocker, artifact,
                    "Kani obligation uses kani::cover! without paired kani::assert or contract postcondition",
                    "Add a kani::assert or function contract postcondition; cover! is non-vacuity evidence only.");
            }
            if patterns.kani_disabled.is_match(cmd) {
                emit(findings, "E_KANI_COVER_ONLY", Severity::Blocker, artifact,
                    "Kani command uses a disabled-check flag (--no-default-checks etc.)",
                    "Remove the disabled-check flag; the obligation is invalid with it.");
            }
            if patterns.kani_hardcoded.is_match(cmd) {
                emit(findings, "E_KANI_ASSUMPTION_VACUITY", Severity::Blocker, artifact,
                    "Kani harness uses hardcoded structural input",
                    "Use kani::any / kani::any_where / kani::bounded_any for symbolic input.");
            }
            if patterns.kani_j_gt_one.is_match(cmd) {
                emit(findings, "E_RESOURCE_UNGOVERNED", Severity::Blocker, artifact,
                    "Kani command uses -j > 1; CBMC memory grows superlinearly",
                    "Use -j 1; record memory headroom only with reviewer acceptance.");
            }
            let mb = row_object(row, "model_bounds");
            let has_high = mb.and_then(|o| o.get("mem_high")).is_some();
            let has_max = mb.and_then(|o| o.get("mem_max")).is_some();
            if !has_high || !has_max {
                emit(findings, "E_RESOURCE_UNGOVERNED", Severity::Blocker, artifact,
                    "Kani model_bounds lacks mem_high / mem_max",
                    "Set mem_high and mem_max; default 20G / 24G with cgroup cap.");
            }
        }
        "loom" => {
            if patterns.loom_std_sync.is_match(cmd) {
                emit(findings, "E_LOOM_TOY_MODEL", Severity::Blocker, artifact,
                    "Loom harness uses std::sync primitives directly instead of crate's sync indirection",
                    "Route through crate's cfg(loom)-aware sync module.");
            }
            let cfg_present = patterns.loom_cfg_lo_present.is_match(cmd);
            if cfg_present && !cmd.contains("--release") {
                emit(findings, "E_LOOM_NOT_MODELED", Severity::Blocker, artifact,
                    "Loom command has --cfg loom but missing --release",
                    "Add --release; Loom without --release is impractical.");
            }
            if cmd.contains("cargo test") && !cfg_present {
                emit(findings, "E_LOOM_NOT_MODELED", Severity::Blocker, artifact,
                    "Loom command missing RUSTFLAGS=--cfg loom",
                    "Add RUSTFLAGS=--cfg loom; the harness will not be modeled otherwise.");
            }
            let mb = row_object(row, "model_bounds");
            let threads_present = mb.and_then(|o| o.get("threads")).is_some();
            if !threads_present {
                emit(findings, "E_PROOF_PLAN_MISSING_NONVACUITY", Severity::Blocker, artifact,
                    "Loom model_bounds lacks threads",
                    "Set threads <= MAX_THREADS=5; record LOOM_MAX_PREEMPTIONS.");
            }
        }
        "miri" => {
            if patterns.miri_present.is_match(cmd) && !patterns.miri_mirflags_present.is_match(cmd) {
                emit(findings, "E_MIRI_TOOLCHAIN_UNPINNED", Severity::Blocker, artifact,
                    "Miri command missing MIRIFLAGS",
                    "Add MIRIFLAGS with at least -Zmiri-strict-provenance and a seed range.");
            }
            let mut tags = BTreeSet::new();
            if let Some(arr) = row_array(row, "risk_tags") {
                for v in arr {
                    if let Some(s) = v.as_str() {
                        tags.insert(s.to_string());
                    }
                }
            }
            const UB_TAGS: &[&str] = &["ub", "unsafe", "ffi", "raw_pointer", "aliasing", "provenance", "layout"];
            let needs_scoping = UB_TAGS.iter().any(|t| tags.contains(*t));
            let tbrs_present = row_array(row, "trusted_base_refs").map(|a| !a.is_empty()).unwrap_or(false);
            if needs_scoping && !tbrs_present {
                emit(findings, "E_MIRI_PATH_SKIPPED", Severity::Blocker, artifact,
                    "Miri obligation for UB/unsafe/ffi/raw_pointer/aliasing/provenance/layout lacks specialist scoping note in trusted-base-plan.md",
                    "Add a Miri Specialist Scoping Note to trusted-base-plan.md and reference it in trusted_base_refs.");
            }
        }
        "flux-rs" if !cmd.contains("nightly") => {
            emit(findings, "E_FLUX_TOOLCHAIN_UNPINNED", Severity::Blocker, artifact,
                "Flux command lacks nightly pin",
                "Pin the nightly date in tool_metadata.version_pin (e.g., 'flux@nightly-2026-02-15').");
        }
        "cargo-fuzz" => {
            if patterns.fuzz_run.is_match(cmd) && !cmd.contains("-max_total_time") {
                emit(findings, "E_FUZZ_BUDGET_ABSENT", Severity::Blocker, artifact,
                    "Fuzz command lacks -max_total_time budget",
                    "Add -max_total_time=120 (smoke) or -max_total_time=1800 (deep).");
            }
            let ev_lower = ev.to_lowercase();
            if !ev_lower.contains("oracle") {
                emit(findings, "E_FUZZ_NO_ORACLE", Severity::Blocker, artifact,
                    "Fuzz obligation's expected_evidence does not name an oracle",
                    "Name an explicit oracle (round-trip, differential, metamorphic, etc.).");
            }
        }
        "proptest" => {
            if patterns.proptest_run.is_match(cmd) && !cmd.contains("PROPTEST_CASES") {
                emit(findings, "E_PROPTEST_BUDGET_ABSENT", Severity::Blocker, artifact,
                    "proptest command lacks PROPTEST_CASES budget",
                    "Add PROPTEST_CASES=10000 minimum.");
            }
            let ev_lower = ev.to_lowercase();
            if !(ev_lower.contains("anti-invariant")
                || ev_lower.contains("anti_invariant")
                || ev_lower.contains("negative")
                || ev_lower.contains("invalid"))
            {
                emit(findings, "E_PROPTEST_NO_ANTI_INVARIANT", Severity::Blocker, artifact,
                    "proptest obligation's expected_evidence does not name an anti-invariant",
                    "Name the anti-invariant or invalid input class that must be rejected.");
            }
        }
        _ => {}
    }
}

fn check_artifact_source(
    row: &Value,
    verifier: &str,
    patterns: &Patterns,
    artifact: &str,
    findings: &mut Vec<Finding>,
) {
    let workdir = match row_str(row, "workdir") {
        Some(w) => Path::new(w),
        None => return,
    };
    if !workdir.is_dir() {
        return;
    }
    let artifact_rel = match row_str(row, "artifact") {
        Some(a) => a,
        None => return,
    };
    let artifact_path = workdir.join(artifact_rel);
    let source = match fs::read_to_string(&artifact_path) {
        Ok(s) => s,
        Err(_) => return,
    };
    let lower = source.to_lowercase();

    match verifier {
        "cargo-fuzz" => {
            let oracle_keys = ["round-trip", "round trip", "differential", "metamorphic",
                "verifier-vs-executor", "verifier vs executor", "encode", "decode"];
            if !oracle_keys.iter().any(|k| lower.contains(k)) {
                emit(findings, "E_FUZZ_NO_ORACLE_IN_SOURCE", Severity::Blocker, artifact,
                    format!("Fuzz harness {artifact_rel} contains no oracle signal in source"),
                    "Add an oracle (round-trip, differential, metamorphic, etc.) to the fuzz harness.");
            }
        }
        "flux-rs" => {
            let negative_keys = ["should_fail", "_negative", "negative_target"];
            let ev_lower = row_str(row, "expected_evidence").unwrap_or("").to_lowercase();
            let ev_has_negative = ev_lower.contains("negative") || ev_lower.contains("should_fail");
            if !negative_keys.iter().any(|k| lower.contains(k)) && !ev_has_negative {
                emit(findings, "E_FLUX_NO_NEGATIVE", Severity::Blocker, artifact,
                    format!("Flux source {artifact_rel} has no paired negative or should_fail target"),
                    "Add a #[should_fail] target or a *_negative function paired with the refinement.");
            }
        }
        "verus" if !lower.contains("exec fn") => {
            emit(findings, "E_VERUS_NO_EXEC_FN", Severity::Blocker, artifact,
                format!("Verus source {artifact_rel} contains no exec fn definition"),
                "Define an exec fn whose body mirrors or calls the production code.");
        }
        "kani" => {
            let assert_keys = ["kani::assert", "#[kani::proof_for_contract]", "#[requires", "#[ensures"];
            if !assert_keys.iter().any(|k| lower.contains(k)) {
                emit(findings, "E_KANI_NO_ASSERTION", Severity::Blocker, artifact,
                    format!("Kani harness {artifact_rel} has no kani::assert or contract postcondition"),
                    "Add a kani::assert or a #[ensures(...)] contract postcondition over the property.");
            }
        }
        "loom" => {
            let sync_keys = ["crate::sync::sync", "loom::sync"];
            if !sync_keys.iter().any(|k| lower.contains(k)) {
                emit(findings, "E_LOOM_NO_SYNC_INDIRECTION", Severity::Blocker, artifact,
                    format!("Loom harness {artifact_rel} does not reference crate::sync::sync::*"),
                    "Route the harness through the crate's cfg(loom)-aware sync indirection.");
            }
        }
        "proptest" => {
            let anti_keys = ["anti_invariant", "anti-invariant", "prop_assume!(false)",
                "should_panic", "invalid_input"];
            if !anti_keys.iter().any(|k| lower.contains(k)) {
                emit(findings, "E_PROPTEST_NO_ANTI_INVARIANT_IN_SOURCE", Severity::Blocker, artifact,
                    format!("proptest source {artifact_rel} has no anti-invariant or invalid-input assertion"),
                    "Add an anti-invariant (e.g., prop_assume!(false), invalid_input) to the proptest strategy.");
            }
        }
        _ => {}
    }
    let _ = patterns;
}

fn check_lane_decisions(
    lane_decisions: &[Value],
    obligations: &[Value],
    findings: &mut Vec<Finding>,
) {
    let mut seen: BTreeSet<(String, String, String, String)> = BTreeSet::new();
    let mut obligation_ids: BTreeSet<String> = BTreeSet::new();
    for o in obligations {
        if let Some(id) = o.get("id").and_then(Value::as_str) {
            obligation_ids.insert(id.to_string());
        }
    }
    for row in lane_decisions {
        if row.get("_parse_error").is_some() {
            emit(findings, "E_SCHEMA_MISSING_FIELD", Severity::Blocker,
                "verifier-lane-decisions.jsonl",
                format!("JSON parse error: {}", row.get("_parse_error").and_then(Value::as_str).unwrap_or("?")),
                "Fix the JSON syntax; every row must be a valid object.",
            );
            continue;
        }
        let id = row_id(row, "?");
        let artifact = format!("verifier-lane-decisions.jsonl::{id}");
        if row_str(row, "schema_version").unwrap_or("") != "verifier-lane-decision/v1" {
            emit(findings, "E_SCHEMA_VERSION_MISSING", Severity::Blocker, &artifact,
                format!("schema_version is {:?}; expected 'verifier-lane-decision/v1'",
                    row_str(row, "schema_version").unwrap_or("")),
                "Set schema_version to the literal 'verifier-lane-decision/v1'.");
            continue;
        }
        for f in LANE_REQUIRED_FIELDS {
            if row.get(*f).is_none() {
                emit(findings, "E_SCHEMA_MISSING_FIELD", Severity::Blocker, &artifact,
                    format!("missing required field {f:?}"),
                    format!("Add the {f:?} field."));
            }
        }
        let applic = row_str(row, "applicability").unwrap_or("");
        if !ALLOWED_APPLICABILITY.contains(&applic) {
            emit(findings, "E_SCHEMA_MISSING_FIELD", Severity::Blocker, &artifact,
                format!("applicability {applic:?} not in {ALLOWED_APPLICABILITY:?}"),
                format!("Use one of {ALLOWED_APPLICABILITY:?}."));
        }
        if applic == "not_applicable" {
            let refs = row_array(row, "non_applicability_evidence_refs").map(|a| !a.is_empty()).unwrap_or(false);
            if !refs {
                emit(findings, "E_LANE_DECISION_WEAK", Severity::Blocker, &artifact,
                    "not_applicable without non_applicability_evidence_refs",
                    "Cite at least one concrete artifact hash plus a typed limitation_kind.");
            }
            let reason = row_str(row, "decision_reason").unwrap_or("").trim();
            let weak = patterns().weak_reason.iter().any(|p| p.is_match(reason));
            if weak {
                emit(findings, "E_LANE_DECISION_WEAK", Severity::Blocker, &artifact,
                    format!("not_applicable reason uses weak vocabulary: {reason:?}"),
                    "Replace with a concrete sentence citing risk_tags and a typed limitation_kind.");
            }
            let limitation = row_str(row, "limitation_kind").unwrap_or("");
            if !ALLOWED_LIMITATION_KIND.contains(&limitation) {
                emit(findings, "E_LANE_DECISION_WEAK", Severity::Blocker, &artifact,
                    format!("limitation_kind {limitation:?} must be one of {ALLOWED_LIMITATION_KIND:?}"),
                    "Pick a typed limitation_kind.");
            }
        }
        if applic == "blocked_tooling" && row_str(row, "tooling_acquisition_ref").map(str::is_empty).unwrap_or(true) {
            emit(findings, "E_LANE_BLOCKED_TOOLING", Severity::Blocker, &artifact,
                "blocked_tooling row lacks tooling_acquisition_ref",
                "Add a tooling_acquisition_ref pointing to a bead or runbook step.");
        }
        if applic == "required" {
            let req_ids = row_array(row, "required_obligation_ids");
            let empty = req_ids.map(|a| a.is_empty()).unwrap_or(true);
            if empty {
                emit(findings, "E_LANE_OBLIGATION_MISSING", Severity::Blocker, &artifact,
                    "required lane decision has no required_obligation_ids",
                    "Name at least one proof-obligation/v1 id.");
            } else if let Some(arr) = req_ids {
                for v in arr {
                    if let Some(s) = v.as_str() {
                        if !obligation_ids.contains(s) {
                            emit(findings, "E_LANE_OBLIGATION_MISSING", Severity::Blocker, &artifact,
                                format!("required obligation id {s:?} not found in proof-obligations.planned.jsonl"),
                                format!("Add an obligation with id {s:?}, or remove the reference."));
                        }
                    }
                }
            }
        }
        let key = (
            row_str(row, "requirement_id").unwrap_or("").to_string(),
            row_str(row, "contract_clause").unwrap_or("").to_string(),
            row_str(row, "proof_seed_id").unwrap_or("").to_string(),
            row_str(row, "verifier").unwrap_or("").to_string(),
        );
        if !seen.insert(key.clone()) {
            emit(findings, "E_LANE_DECISION_DUPLICATE", Severity::Blocker, &artifact,
                format!("duplicate lane decision for {key:?}"),
                "Remove the duplicate; one lane decision per (req, cc, seed, verifier) tuple.");
        }
    }
    let mut obligation_to_lanes: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for row in lane_decisions {
        if row_str(row, "applicability").unwrap_or("") == "required" {
            let lane_id = row_str(row, "id").unwrap_or("?").to_string();
            if let Some(arr) = row_array(row, "required_obligation_ids") {
                for v in arr {
                    if let Some(s) = v.as_str() {
                        obligation_to_lanes.entry(s.to_string()).or_default().push(lane_id.clone());
                    }
                }
            }
        }
    }
    for row in obligations {
        if row.get("_parse_error").is_some() {
            continue;
        }
        if let Some(oid) = row.get("id").and_then(Value::as_str) {
            if !obligation_to_lanes.contains_key(oid) {
                let artifact = format!("proof-obligations.planned.jsonl::{oid}");
                emit(findings, "E_LANE_OBLIGATION_MISSING", Severity::Blocker, &artifact,
                    format!("obligation {oid:?} is not referenced by any required lane decision"),
                    "Add a verifier-lane-decision/v1 row with applicability=required and required_obligation_ids including this id.");
            }
        }
    }
}

fn check_waivers(waivers: &[Value], findings: &mut Vec<Finding>) {
    for row in waivers {
        if row.get("_parse_error").is_some() {
            emit(findings, "E_SCHEMA_MISSING_FIELD", Severity::Blocker,
                "waiver-candidates.jsonl",
                format!("JSON parse error: {}", row.get("_parse_error").and_then(Value::as_str).unwrap_or("?")),
                "Fix the JSON syntax.");
            continue;
        }
        let id = row_id(row, "?");
        let artifact = format!("waiver-candidates.jsonl::{id}");
        if row_str(row, "schema_version").unwrap_or("") != "waiver-candidate/v1" {
            emit(findings, "E_SCHEMA_VERSION_MISSING", Severity::Blocker, &artifact,
                format!("schema_version is {:?}; expected 'waiver-candidate/v1'",
                    row_str(row, "schema_version").unwrap_or("")),
                "Set schema_version to the literal 'waiver-candidate/v1'.");
            continue;
        }
        for f in WAIVER_REQUIRED_FIELDS {
            if row.get(*f).is_none() {
                emit(findings, "E_WAIVER_LIFECYCLE_INVALID", Severity::Blocker, &artifact,
                    format!("missing required field {f:?}"),
                    format!("Add the {f:?} field."));
            }
        }
        if row_bool(row, "behavior_affecting").unwrap_or(false) {
            emit(findings, "E_BEHAVIOR_WAIVER", Severity::Blocker, &artifact,
                "behavior-affecting waiver is invalid",
                "Behavior-affecting waivers are forbidden; route to a debt bead instead.");
        }
        let bp = row_str(row, "boundary_proof").unwrap_or("").trim();
        if bp.is_empty() {
            emit(findings, "E_WAIVER_LIFECYCLE_INVALID", Severity::Blocker, &artifact,
                "boundary_proof is empty",
                "Cite a sibling obligation, verification-ledger hash, or upstream spec.");
        }
        let ce = row_array(row, "compensating_evidence");
        if ce.map(|a| a.is_empty()).unwrap_or(true) {
            emit(findings, "E_WAIVER_LIFECYCLE_INVALID", Severity::Blocker, &artifact,
                "compensating_evidence is empty",
                "Provide at least one compensating_evidence reference.");
        }
        let reason = row_str(row, "reason").unwrap_or("").trim();
        if patterns().weak_reason.iter().any(|p| p.is_match(reason)) {
            emit(findings, "E_WAIVER_LIFECYCLE_INVALID", Severity::Blocker, &artifact,
                format!("reason uses weak vocabulary: {reason:?}"),
                "Replace with a concrete sentence citing the limitation_kind.");
        }
        let expiry = row_str(row, "expiry").unwrap_or("").trim();
        if !patterns().iso8601_prefix.is_match(expiry) {
            emit(findings, "E_WAIVER_LIFECYCLE_INVALID", Severity::Blocker, &artifact,
                format!("expiry {expiry:?} is not ISO-8601"),
                "Set expiry to an ISO-8601 date (e.g., '2026-12-31').");
        }
    }
}

/// A lane-review row counts as reviewer acceptance only when an independent
/// reviewer invocation accepted the exact lane decision. Self-stamped rows
/// (planner and reviewer invocation ids equal, or either missing) never
/// count: planner artifacts must not self-stamp reviewer disposition.
fn lane_review_accepts(lane_reviews: &[Value], lane_decision_id: &str) -> bool {
    lane_reviews.iter().any(|r| {
        if r.get("_parse_error").is_some() {
            return false;
        }
        if row_str(r, "schema_version").unwrap_or("") != "verifier-lane-review/v1" {
            return false;
        }
        if row_str(r, "lane_decision_id").unwrap_or("") != lane_decision_id {
            return false;
        }
        if row_str(r, "reviewer_disposition").unwrap_or("") != "accepted" {
            return false;
        }
        let planner = row_str(r, "planner_invocation_id").unwrap_or("");
        let reviewer = row_str(r, "reviewer_invocation_id").unwrap_or("");
        !planner.is_empty() && !reviewer.is_empty() && planner != reviewer
    })
}

/// True when `s` carries a concrete SHA-256 artifact hash: 64 consecutive
/// hexadecimal characters, optionally prefixed (e.g. `SHA-256:<hex>`).
fn contains_sha256_hex(s: &str) -> bool {
    let bytes = s.as_bytes();
    let mut run = 0usize;
    for b in bytes {
        if b.is_ascii_hexdigit() {
            run += 1;
            if run >= 64 {
                return true;
            }
        } else {
            run = 0;
        }
    }
    false
}

/// Index of `(requirement_id, contract_clause, verifier)` tuples whose
/// default-profile obligation is excused by canonical lane policy: a
/// `not_applicable` lane decision with concrete hashed evidence, a typed
/// `limitation_kind`, a concrete reason, and independent reviewer
/// acceptance. Anything weaker (missing refs, unhashed refs, weak reason,
/// missing/typed-wrong limitation, no accepted review) is absent from the
/// index, so the coverage check keeps flagging it.
fn accepted_na_lanes(
    lane_decisions: &[Value],
    lane_reviews: &[Value],
) -> BTreeSet<(String, String, String)> {
    let mut out = BTreeSet::new();
    for row in lane_decisions {
        if row.get("_parse_error").is_some() {
            continue;
        }
        if row_str(row, "schema_version").unwrap_or("") != "verifier-lane-decision/v1" {
            continue;
        }
        if row_str(row, "applicability").unwrap_or("") != "not_applicable" {
            continue;
        }
        let verifier = row_str(row, "verifier").unwrap_or("");
        if !ALLOWED_VERIFIERS.contains(&verifier) {
            continue;
        }
        let hashed = row_array(row, "non_applicability_evidence_refs").is_some_and(|a| {
            a.iter()
                .any(|v| v.as_str().map(contains_sha256_hex).unwrap_or(false))
        });
        if !hashed {
            continue;
        }
        if !ALLOWED_LIMITATION_KIND.contains(&row_str(row, "limitation_kind").unwrap_or("")) {
            continue;
        }
        let reason = row_str(row, "decision_reason").unwrap_or("").trim();
        if reason.is_empty()
            || patterns().weak_reason.iter().any(|p| p.is_match(reason))
        {
            continue;
        }
        let id = row_id(row, "");
        if id.is_empty() || !lane_review_accepts(lane_reviews, id) {
            continue;
        }
        out.insert((
            row_str(row, "requirement_id").unwrap_or("").to_string(),
            row_str(row, "contract_clause").unwrap_or("").to_string(),
            verifier.to_string(),
        ));
    }
    out
}

fn check_risk_profile_coverage(
    obligations: &[Value],
    lane_decisions: &[Value],
    lane_reviews: &[Value],
    findings: &mut Vec<Finding>,
) {
    let mut groups: BTreeMap<(String, String), (BTreeSet<String>, BTreeSet<String>)> = BTreeMap::new();
    for row in obligations {
        if row.get("_parse_error").is_some() {
            continue;
        }
        let req = row_str(row, "requirement_id").unwrap_or("").to_string();
        let cc = row_str(row, "contract_clause").unwrap_or("").to_string();
        let verifier = row_str(row, "verifier").unwrap_or("");
        let risk = row_str(row, "risk").unwrap_or("").to_string();
        let entry = groups.entry((req, cc)).or_insert_with(|| (BTreeSet::new(), BTreeSet::new()));
        entry.0.insert(risk);
        if !verifier.is_empty() && ALLOWED_VERIFIERS.contains(&verifier) {
            entry.1.insert(verifier.to_string());
        }
    }
    let na_accepted = accepted_na_lanes(lane_decisions, lane_reviews);
    let mut reported: BTreeSet<(String, String, String, String)> = BTreeSet::new();
    for ((req, cc), (risks, verifiers)) in &groups {
        for risk in risks {
            let required: &[&'static str] = match risk_profile_required(risk) {
                Some(r) => r,
                None => continue,
            };
            for rv in required {
                let key = (req.clone(), cc.clone(), risk.clone(), (*rv).to_string());
                if reported.contains(&key) {
                    continue;
                }
                reported.insert(key);
                if !verifiers.contains(*rv)
                    && !na_accepted.contains(&(req.clone(), cc.clone(), (*rv).to_string()))
                {
                    let artifact = format!("({req}, {cc}, {risk})");
                    emit(findings, "E_LANE_DECISION_MISSING", Severity::Major, &artifact,
                        format!("default-profile verifier {rv:?} missing for risk {risk:?}"),
                        format!("Add an obligation for {rv:?}, a waiver citing concrete evidence, \
                            or an accepted not_applicable lane decision with hashed evidence refs, \
                            a typed limitation_kind, and an independent accepted lane review."));
                }
            }
        }
    }
}

pub fn patterns() -> &'static Patterns {
    use std::sync::OnceLock;
    static P: OnceLock<Patterns> = OnceLock::new();
    P.get_or_init(Patterns::new)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use serde_json::json;

    // ---- check_target_shape ----

    #[test]
    fn target_shape_accepts_crate_path_symbol() {
        assert!(check_target_shape("crate::counter::Counter::tick"));
    }

    #[test]
    fn target_shape_accepts_file_path_symbol() {
        assert!(check_target_shape("crates/parser/src/lib.rs::parser::Frame::decode"));
    }

    #[test]
    fn target_shape_rejects_bare_file_path() {
        assert!(!check_target_shape("src/lib.rs"));
        assert!(!check_target_shape("crates/parser/src/lib.rs"));
    }

    #[test]
    fn target_shape_rejects_empty_string() {
        assert!(!check_target_shape(""));
    }

    // ---- check_command_specificity ----

    #[test]
    fn command_specificity_accepts_named_harness() {
        assert!(check_command_specificity("cargo kani -j 1 --harness frame_decode_rejects_truncation"));
    }

    #[test]
    fn command_specificity_rejects_bare_cargo_test() {
        assert!(!check_command_specificity("cargo test"));
        assert!(!check_command_specificity("cargo test -p foo"));
    }

    #[test]
    fn command_specificity_rejects_bare_cargo_build() {
        assert!(!check_command_specificity("cargo build"));
    }

    #[test]
    fn command_specificity_rejects_placeholders() {
        assert!(!check_command_specificity("cargo kani --harness <NAME>"));
        assert!(!check_command_specificity("cargo kani --harness ..."));
    }

    #[test]
    fn command_specificity_rejects_empty() {
        assert!(!check_command_specificity(""));
    }

    // ---- check_tool_marker ----

    #[test]
    fn tool_marker_accepts_kani_success() {
        assert!(check_tool_marker("Kani reports VERIFICATION:- SUCCESSFUL for foo"));
    }

    #[test]
    fn tool_marker_accepts_test_result_ok() {
        assert!(check_tool_marker("cargo test passes with test result: ok. 5 passed"));
    }

    #[test]
    fn tool_marker_accepts_verus_succeeded() {
        assert!(check_tool_marker("Verus reports Verification SUCCEEDED"));
    }

    #[test]
    fn tool_marker_accepts_model_checking_completed() {
        assert!(check_tool_marker("TLC reports Model checking completed"));
    }

    #[test]
    fn tool_marker_accepts_no_ub_diagnostic() {
        assert!(check_tool_marker("Miri reports no UB diagnostic"));
    }

    #[test]
    fn tool_marker_rejects_vague_evidence() {
        assert!(!check_tool_marker("the verifier ran and everything looks good"));
        assert!(!check_tool_marker("it works"));
        assert!(!check_tool_marker(""));
    }

    // ---- ALLOWED_VERIFIERS and constants ----

    #[test]
    fn allowed_verifiers_excludes_tla_plus() {
        assert!(!ALLOWED_VERIFIERS.contains(&"tla-plus"),
            "TLA+ must be removed from this skill; found in ALLOWED_VERIFIERS");
    }

    #[test]
    fn allowed_verifiers_has_seven_entries() {
        assert_eq!(ALLOWED_VERIFIERS.len(), 7);
    }

    #[test]
    fn allowed_verifiers_contains_the_seven_canonical_verifiers() {
        for v in ["verus", "kani", "flux-rs", "loom", "miri", "cargo-fuzz", "proptest"] {
            assert!(ALLOWED_VERIFIERS.contains(&v), "missing verifier: {v}");
        }
    }

    #[test]
    fn default_risk_profile_does_not_require_tla_plus() {
        for (_, lanes) in DEFAULT_RISK_PROFILE {
            for v in *lanes {
                assert_ne!(*v, "tla-plus",
                    "DEFAULT_RISK_PROFILE must not require tla-plus; found in {lanes:?}");
            }
        }
    }

    #[test]
    fn risk_profile_required_returns_lanes_for_known_risk() {
        let lanes = risk_profile_required("arithmetic_overflow").expect("must be known risk");
        assert!(lanes.contains(&"verus"));
        assert!(lanes.contains(&"kani"));
        assert!(lanes.contains(&"proptest"));
        assert_eq!(lanes.len(), 3);
    }

    #[test]
    fn risk_profile_required_returns_none_for_unknown_risk() {
        assert!(risk_profile_required("nonexistent_risk_class").is_none());
    }

    #[test]
    fn temporal_liveness_profile_is_loom_and_proptest() {
        let lanes = risk_profile_required("temporal_liveness").expect("must be known risk");
        assert_eq!(*lanes, ["loom", "proptest"]);
    }

    #[test]
    fn temporal_safety_profile_is_verus_and_loom() {
        let lanes = risk_profile_required("temporal_safety").expect("must be known risk");
        assert_eq!(*lanes, ["verus", "loom"]);
    }

    // ---- load_jsonl ----

    #[test]
    fn load_jsonl_reads_one_object_per_line() {
        let dir = tempdir();
        let path = dir.join("plan.jsonl");
        std::fs::write(&path, "{\"id\":\"a\"}\n{\"id\":\"b\"}\n\n{\"id\":\"c\"}\n").unwrap();
        let rows = load_jsonl(&path).unwrap();
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].get("id").and_then(Value::as_str), Some("a"));
        assert_eq!(rows[2].get("id").and_then(Value::as_str), Some("c"));
    }

    #[test]
    fn load_jsonl_records_parse_errors_with_line_numbers() {
        let dir = tempdir();
        let path = dir.join("bad.jsonl");
        std::fs::write(&path, "{\"id\":\"a\"}\nnot-json\n{\"id\":\"c\"}\n").unwrap();
        let rows = load_jsonl(&path).unwrap();
        assert_eq!(rows.len(), 3);
        assert!(rows[0].get("id").is_some());
        assert_eq!(rows[1].get("_parse_error").and_then(Value::as_str), Some("line 2"));
        assert_eq!(rows[1].get("_line").and_then(Value::as_u64), Some(2));
        assert!(rows[2].get("id").is_some());
    }

    #[test]
    fn load_jsonl_returns_empty_vec_for_missing_file() {
        let path = Path::new("/nonexistent/path/file.jsonl");
        let rows = load_jsonl(path).unwrap();
        assert!(rows.is_empty());
    }

    #[test]
    fn load_jsonl_returns_empty_vec_for_empty_file() {
        let dir = tempdir();
        let path = dir.join("empty.jsonl");
        std::fs::write(&path, "").unwrap();
        let rows = load_jsonl(&path).unwrap();
        assert!(rows.is_empty());
    }

    // ---- Patterns ----

    #[test]
    fn patterns_new_compiles_all_regexes() {
        let _p = Patterns::new();
        // No assertion needed; if a regex fails to compile, the test fails.
    }

    #[test]
    fn patterns_default_via_trait_equals_new() {
        let a = Patterns::new();
        let b = Patterns::default();
        let ra = a.kani_disabled.as_str();
        let rb = b.kani_disabled.as_str();
        assert_eq!(ra, rb);
    }

    // ---- run_all_checks: happy path on minimal plan ----

    #[test]
    fn run_all_checks_produces_zero_findings_on_empty_plan() {
        let plan = Plan::default();
        let mut findings = Vec::new();
        run_all_checks(&plan, &Default::default(), &mut findings);
        assert!(findings.is_empty(), "empty plan must yield zero findings");
    }

    #[test]
    fn run_all_checks_emits_unpaired_obligation_finding() {
        let plan = Plan {
            obligations: vec![json!({
                "schema_version": "proof-obligation/v1",
                "id": "PO-001",
                "requirement_id": "REQ-1",
                "contract_clause": "C-1",
                "domain_claim": "x",
                "risk": "arithmetic_overflow",
                "risk_tags": ["arithmetic"],
                "verifier": "verus",
                "artifact": "src/lib.rs",
                "target": "crate::x::y",
                "command": "verus src/lib.rs",
                "workdir": "/abs",
                "expected_evidence": "Verification SUCCEEDED",
                "assumptions": [],
                "model_bounds": {},
                "tool_metadata": {"tool": "verus", "version_pin": "verus@0.2024.10"},
                "trusted_base_refs": [],
                "required": true,
                "behavior_affecting": true,
                "mode": "verify-proof",
                "owner_state": 4,
                "rerun_from": 4,
                "status": "planned",
            })],
            lane_decisions: vec![],
            waivers: vec![],
            lane_reviews: vec![],
        };
        let mut findings = Vec::new();
        run_all_checks(&plan, &Default::default(), &mut findings);
        let codes: Vec<&str> = findings.iter().map(|f| f.finding_code.as_str()).collect();
        assert!(codes.contains(&"E_LANE_OBLIGATION_MISSING"),
            "expected E_LANE_OBLIGATION_MISSING; got: {codes:?}");
    }

    #[test]
    fn run_all_checks_emits_e_behavior_waiver_for_behavior_affecting_true() {
        let plan = Plan {
            obligations: vec![],
            lane_decisions: vec![],
            waivers: vec![json!({
                "schema_version": "waiver-candidate/v1",
                "id": "WVR-001",
                "requirement_id": "REQ-1",
                "contract_clause": "C-1",
                "reason": "waiving for behavior X",
                "behavior_affecting": true,
                "boundary_proof": "",
                "compensating_evidence": [],
                "owner": "",
                "expiry": "yesterday",
                "review_status": "proposed",
            })],
            ..Plan::default()
        };
        let mut findings = Vec::new();
        run_all_checks(&plan, &Default::default(), &mut findings);
        let codes: Vec<&str> = findings.iter().map(|f| f.finding_code.as_str()).collect();
        assert!(codes.contains(&"E_BEHAVIOR_WAIVER"),
            "expected E_BEHAVIOR_WAIVER; got: {codes:?}");
        assert!(codes.contains(&"E_WAIVER_LIFECYCLE_INVALID"),
            "expected E_WAIVER_LIFECYCLE_INVALID for empty boundary_proof / bad expiry");
    }

    #[test]
    fn run_all_checks_emits_e_verus_disconnected_spec_for_assume() {
        let plan = Plan {
            obligations: vec![json!({
                "schema_version": "proof-obligation/v1",
                "id": "PO-001",
                "requirement_id": "REQ-1",
                "contract_clause": "C-1",
                "domain_claim": "x",
                "risk": "arithmetic_overflow",
                "risk_tags": ["arithmetic"],
                "verifier": "verus",
                "artifact": "src/lib.rs",
                "target": "crate::x::y",
                "command": "verus src/lib.rs",
                "workdir": "/abs",
                "expected_evidence": "Verification SUCCEEDED",
                "assumptions": [],
                "model_bounds": {},
                "tool_metadata": {"tool": "verus", "version_pin": "verus@0.2024.10"},
                "trusted_base_refs": [],
                "required": true,
                "behavior_affecting": true,
                "mode": "verify-proof",
                "owner_state": 4,
                "rerun_from": 4,
                "status": "planned",
            })],
            ..Plan::default()
        };
        let mut findings = Vec::new();
        run_all_checks(&plan, &Default::default(), &mut findings);
        let codes: Vec<&str> = findings.iter().map(|f| f.finding_code.as_str()).collect();
        assert!(!codes.contains(&"E_VERUS_DISCONNECTED_SPEC"),
            "verus with no external_body/assume in command+target must NOT trigger E_VERUS_DISCONNECTED_SPEC");
    }

    #[test]
    fn run_all_checks_emits_e_verus_disconnected_spec_when_command_contains_assume() {
        let plan = Plan {
            obligations: vec![json!({
                "schema_version": "proof-obligation/v1",
                "id": "PO-001",
                "requirement_id": "REQ-1",
                "contract_clause": "C-1",
                "domain_claim": "x",
                "risk": "arithmetic_overflow",
                "risk_tags": ["arithmetic"],
                "verifier": "verus",
                "artifact": "src/lib.rs",
                "target": "crate::x::y",
                "command": "verus src/lib.rs # assume(x); axiom y",
                "workdir": "/abs",
                "expected_evidence": "Verification SUCCEEDED",
                "assumptions": [],
                "model_bounds": {},
                "tool_metadata": {"tool": "verus", "version_pin": "verus@0.2024.10"},
                "trusted_base_refs": [],
                "required": true,
                "behavior_affecting": true,
                "mode": "verify-proof",
                "owner_state": 4,
                "rerun_from": 4,
                "status": "planned",
            })],
            ..Plan::default()
        };
        let mut findings = Vec::new();
        run_all_checks(&plan, &Default::default(), &mut findings);
        let codes: Vec<&str> = findings.iter().map(|f| f.finding_code.as_str()).collect();
        assert!(codes.contains(&"E_VERUS_DISCONNECTED_SPEC"),
            "expected E_VERUS_DISCONNECTED_SPEC when command has assume/axiom; got: {codes:?}");
    }

    #[test]
    fn run_all_checks_emits_e_kani_cover_only_when_only_cover_present() {
        let plan = Plan {
            obligations: vec![json!({
                "schema_version": "proof-obligation/v1",
                "id": "PO-001",
                "requirement_id": "REQ-1",
                "contract_clause": "C-1",
                "domain_claim": "x",
                "risk": "rejection",
                "risk_tags": ["rejection"],
                "verifier": "kani",
                "artifact": "src/lib.rs",
                "target": "crate::x::y",
                "command": "cargo kani -j 1 --harness foo # cover!(true)",
                "workdir": "/abs",
                "expected_evidence": "Kani VERIFICATION:- SUCCESSFUL",
                "assumptions": [],
                "model_bounds": {"unwind": 4, "mem_high": "20G", "mem_max": "24G"},
                "tool_metadata": {"tool": "cargo-kani", "version_pin": "cargo-kani@0.65"},
                "trusted_base_refs": [],
                "required": true,
                "behavior_affecting": true,
                "mode": "verify-proof",
                "owner_state": 4,
                "rerun_from": 4,
                "status": "planned",
            })],
            ..Plan::default()
        };
        let mut findings = Vec::new();
        run_all_checks(&plan, &Default::default(), &mut findings);
        let codes: Vec<&str> = findings.iter().map(|f| f.finding_code.as_str()).collect();
        assert!(codes.contains(&"E_KANI_COVER_ONLY"),
            "expected E_KANI_COVER_ONLY when kani::cover! without assert/contract; got: {codes:?}");
    }

    #[test]
    fn run_all_checks_emits_e_resource_ungoverned_for_j_gt_one() {
        let plan = Plan {
            obligations: vec![json!({
                "schema_version": "proof-obligation/v1",
                "id": "PO-001",
                "requirement_id": "REQ-1",
                "contract_clause": "C-1",
                "domain_claim": "x",
                "risk": "rejection",
                "risk_tags": ["rejection"],
                "verifier": "kani",
                "artifact": "src/lib.rs",
                "target": "crate::x::y",
                "command": "cargo kani -j 4 --harness foo",
                "workdir": "/abs",
                "expected_evidence": "VERIFICATION:- SUCCESSFUL",
                "assumptions": [],
                "model_bounds": {"unwind": 4, "mem_high": "20G", "mem_max": "24G"},
                "tool_metadata": {"tool": "cargo-kani", "version_pin": "cargo-kani@0.65"},
                "trusted_base_refs": [],
                "required": true,
                "behavior_affecting": true,
                "mode": "verify-proof",
                "owner_state": 4,
                "rerun_from": 4,
                "status": "planned",
            })],
            ..Plan::default()
        };
        let mut findings = Vec::new();
        run_all_checks(&plan, &Default::default(), &mut findings);
        let codes: Vec<&str> = findings.iter().map(|f| f.finding_code.as_str()).collect();
        assert!(codes.contains(&"E_RESOURCE_UNGOVERNED"),
            "expected E_RESOURCE_UNGOVERNED for -j 4; got: {codes:?}");
    }

    #[test]
    fn run_all_checks_emits_e_loom_not_modeled_when_cfg_loom_missing() {
        let plan = Plan {
            obligations: vec![json!({
                "schema_version": "proof-obligation/v1",
                "id": "PO-001",
                "requirement_id": "REQ-1",
                "contract_clause": "C-1",
                "domain_claim": "x",
                "risk": "concurrency_interleaving",
                "risk_tags": ["concurrency"],
                "verifier": "loom",
                "artifact": "src/lib.rs",
                "target": "crate::q::Q",
                "command": "cargo test --test mpsc_drain --release",
                "workdir": "/abs",
                "expected_evidence": "Loom test result: ok",
                "assumptions": [],
                "model_bounds": {"threads": 4},
                "tool_metadata": {"tool": "loom", "version_pin": "loom@0.7"},
                "trusted_base_refs": [],
                "required": true,
                "behavior_affecting": true,
                "mode": "verify-proof",
                "owner_state": 4,
                "rerun_from": 4,
                "status": "planned",
            })],
            ..Plan::default()
        };
        let mut findings = Vec::new();
        run_all_checks(&plan, &Default::default(), &mut findings);
        let codes: Vec<&str> = findings.iter().map(|f| f.finding_code.as_str()).collect();
        assert!(codes.contains(&"E_LOOM_NOT_MODELED"),
            "expected E_LOOM_NOT_MODELED when --cfg loom absent; got: {codes:?}");
    }

    #[test]
    fn run_all_checks_emits_e_miri_path_skipped_for_ub_tags_without_trusted_base() {
        let plan = Plan {
            obligations: vec![json!({
                "schema_version": "proof-obligation/v1",
                "id": "PO-001",
                "requirement_id": "REQ-1",
                "contract_clause": "C-1",
                "domain_claim": "x",
                "risk": "ub_safety",
                "risk_tags": ["unsafe", "raw_pointer"],
                "verifier": "miri",
                "artifact": "src/lib.rs",
                "target": "crate::a::alloc",
                "command": "MIRIFLAGS=\"-Zmiri-strict-provenance\" cargo +nightly miri test --test alloc --release",
                "workdir": "/abs",
                "expected_evidence": "no UB diagnostic",
                "assumptions": [],
                "model_bounds": {},
                "tool_metadata": {"tool": "cargo-miri", "version_pin": "nightly-2026-02-15"},
                "trusted_base_refs": [],
                "required": true,
                "behavior_affecting": true,
                "mode": "verify-proof",
                "owner_state": 4,
                "rerun_from": 4,
                "status": "planned",
            })],
            ..Plan::default()
        };
        let mut findings = Vec::new();
        run_all_checks(&plan, &Default::default(), &mut findings);
        let codes: Vec<&str> = findings.iter().map(|f| f.finding_code.as_str()).collect();
        assert!(codes.contains(&"E_MIRI_PATH_SKIPPED"),
            "expected E_MIRI_PATH_SKIPPED when UB tags present without trusted_base_refs; got: {codes:?}");
    }

    #[test]
    fn run_all_checks_emits_e_fuzz_budget_absent_and_no_oracle() {
        let plan = Plan {
            obligations: vec![json!({
                "schema_version": "proof-obligation/v1",
                "id": "PO-001",
                "requirement_id": "REQ-1",
                "contract_clause": "C-1",
                "domain_claim": "x",
                "risk": "hostile_input",
                "risk_tags": ["parser"],
                "verifier": "cargo-fuzz",
                "artifact": "src/lib.rs",
                "target": "crate::p::p",
                "command": "cargo fuzz run p --sanitizer=address",
                "workdir": "/abs",
                "expected_evidence": "fuzz runs",
                "assumptions": [],
                "model_bounds": {},
                "tool_metadata": {"tool": "cargo-fuzz", "version_pin": "cargo-fuzz@0.13"},
                "trusted_base_refs": [],
                "required": true,
                "behavior_affecting": true,
                "mode": "verify-proof",
                "owner_state": 4,
                "rerun_from": 4,
                "status": "planned",
            })],
            ..Plan::default()
        };
        let mut findings = Vec::new();
        run_all_checks(&plan, &Default::default(), &mut findings);
        let codes: Vec<&str> = findings.iter().map(|f| f.finding_code.as_str()).collect();
        assert!(codes.contains(&"E_FUZZ_BUDGET_ABSENT"));
        assert!(codes.contains(&"E_FUZZ_NO_ORACLE"));
        assert!(codes.contains(&"E_PROOF_PLAN_MISSING_NONVACUITY"));
    }

    #[test]
    fn run_all_checks_emits_e_proptest_budget_and_anti_invariant_absent() {
        let plan = Plan {
            obligations: vec![json!({
                "schema_version": "proof-obligation/v1",
                "id": "PO-001",
                "requirement_id": "REQ-1",
                "contract_clause": "C-1",
                "domain_claim": "x",
                "risk": "ordering",
                "risk_tags": ["property"],
                "verifier": "proptest",
                "artifact": "src/lib.rs",
                "target": "crate::s::sort",
                "command": "cargo test --test sort --release",
                "workdir": "/abs",
                "expected_evidence": "proptest passes",
                "assumptions": [],
                "model_bounds": {},
                "tool_metadata": {"tool": "proptest", "version_pin": "proptest@1.5"},
                "trusted_base_refs": [],
                "required": true,
                "behavior_affecting": true,
                "mode": "verify-proof",
                "owner_state": 4,
                "rerun_from": 4,
                "status": "planned",
            })],
            ..Plan::default()
        };
        let mut findings = Vec::new();
        run_all_checks(&plan, &Default::default(), &mut findings);
        let codes: Vec<&str> = findings.iter().map(|f| f.finding_code.as_str()).collect();
        assert!(codes.contains(&"E_PROPTEST_BUDGET_ABSENT"));
        assert!(codes.contains(&"E_PROPTEST_NO_ANTI_INVARIANT"));
    }

    #[test]
    fn run_all_checks_emits_e_flux_toolchain_unpinned() {
        let plan = Plan {
            obligations: vec![json!({
                "schema_version": "proof-obligation/v1",
                "id": "PO-001",
                "requirement_id": "REQ-1",
                "contract_clause": "C-1",
                "domain_claim": "x",
                "risk": "refinement",
                "risk_tags": ["refinement"],
                "verifier": "flux-rs",
                "artifact": "src/lib.rs",
                "target": "crate::i::new",
                "command": "cargo flux --lib",
                "workdir": "/abs",
                "expected_evidence": "Flux check passed",
                "assumptions": [],
                "model_bounds": {},
                "tool_metadata": {"tool": "cargo-flux", "version_pin": "flux@latest"},
                "trusted_base_refs": [],
                "required": true,
                "behavior_affecting": true,
                "mode": "verify-proof",
                "owner_state": 4,
                "rerun_from": 4,
                "status": "planned",
            })],
            ..Plan::default()
        };
        let mut findings = Vec::new();
        run_all_checks(&plan, &Default::default(), &mut findings);
        let codes: Vec<&str> = findings.iter().map(|f| f.finding_code.as_str()).collect();
        assert!(codes.contains(&"E_FLUX_TOOLCHAIN_UNPINNED"));
    }

    #[test]
    fn run_all_checks_emits_e_schema_version_missing_for_wrong_version() {
        let plan = Plan {
            obligations: vec![json!({
                "schema_version": "wrong-version",
                "id": "PO-001",
                "requirement_id": "REQ-1",
                "contract_clause": "C-1",
                "domain_claim": "x",
                "risk": "arithmetic_overflow",
                "risk_tags": ["arithmetic"],
                "verifier": "verus",
                "artifact": "src/lib.rs",
                "target": "crate::x::y",
                "command": "verus src/lib.rs",
                "workdir": "/abs",
                "expected_evidence": "Verification SUCCEEDED",
                "assumptions": [],
                "model_bounds": {},
                "tool_metadata": {"tool": "verus", "version_pin": "verus@0.2024.10"},
                "trusted_base_refs": [],
                "required": true,
                "behavior_affecting": true,
                "mode": "verify-proof",
                "owner_state": 4,
                "rerun_from": 4,
                "status": "planned",
            })],
            ..Plan::default()
        };
        let mut findings = Vec::new();
        run_all_checks(&plan, &Default::default(), &mut findings);
        let codes: Vec<&str> = findings.iter().map(|f| f.finding_code.as_str()).collect();
        assert!(codes.contains(&"E_SCHEMA_VERSION_MISSING"));
    }

    #[test]
    fn run_all_checks_emits_e_lane_decision_duplicate_for_duplicate_keys() {
        let plan = Plan {
            obligations: vec![json!({
                "schema_version": "proof-obligation/v1",
                "id": "PO-001",
                "requirement_id": "REQ-1",
                "contract_clause": "C-1",
                "domain_claim": "x",
                "risk": "arithmetic_overflow",
                "risk_tags": ["arithmetic"],
                "verifier": "verus",
                "artifact": "src/lib.rs",
                "target": "crate::x::y",
                "command": "verus src/lib.rs",
                "workdir": "/abs",
                "expected_evidence": "Verification SUCCEEDED",
                "assumptions": [],
                "model_bounds": {},
                "tool_metadata": {"tool": "verus", "version_pin": "verus@0.2024.10"},
                "trusted_base_refs": [],
                "required": true,
                "behavior_affecting": true,
                "mode": "verify-proof",
                "owner_state": 4,
                "rerun_from": 4,
                "status": "planned",
            })],
            lane_decisions: vec![
                json!({
                    "schema_version": "verifier-lane-decision/v1",
                    "id": "VLD-001",
                    "requirement_id": "REQ-1",
                    "contract_clause": "C-1",
                    "proof_seed_id": "PS-1",
                    "verifier": "verus",
                    "risk_tags": ["arithmetic"],
                    "applicability": "required",
                    "decision_reason": "ok",
                    "required_obligation_ids": ["PO-001"],
                    "non_applicability_evidence_refs": [],
                    "limitation_kind": "",
                    "owner_state": 4,
                    "status": "planned",
                }),
                json!({
                    "schema_version": "verifier-lane-decision/v1",
                    "id": "VLD-002",
                    "requirement_id": "REQ-1",
                    "contract_clause": "C-1",
                    "proof_seed_id": "PS-1",
                    "verifier": "verus",
                    "risk_tags": ["arithmetic"],
                    "applicability": "required",
                    "decision_reason": "ok",
                    "required_obligation_ids": ["PO-001"],
                    "non_applicability_evidence_refs": [],
                    "limitation_kind": "",
                    "owner_state": 4,
                    "status": "planned",
                }),
            ],
            waivers: vec![],
            lane_reviews: vec![],
        };
        let mut findings = Vec::new();
        run_all_checks(&plan, &Default::default(), &mut findings);
        let codes: Vec<&str> = findings.iter().map(|f| f.finding_code.as_str()).collect();
        assert!(codes.contains(&"E_LANE_DECISION_DUPLICATE"));
    }

    // ---- check_risk_profile_coverage vs accepted not_applicable lanes ----
    //
    // vb-3s3x9: the coverage check must honor concrete hashed
    // not_applicable lane decisions with independent reviewer acceptance,
    // while keeping every negative case (missing/unjustified N/A,
    // changed-behavior lanes).

    const TEST_HASH: &str =
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

    fn na_lane(id: &str, verifier: &str, refs: Vec<Value>, kind: &str, reason: &str) -> Value {
        json!({
            "schema_version": "verifier-lane-decision/v1",
            "id": id,
            "requirement_id": "REQ-M",
            "contract_clause": "C-MIG",
            "proof_seed_id": "PS-M",
            "verifier": verifier,
            "risk_tags": ["migration"],
            "applicability": "not_applicable",
            "decision_reason": reason,
            "required_obligation_ids": [],
            "non_applicability_evidence_refs": refs,
            "limitation_kind": kind,
            "owner_state": 4,
            "status": "planned",
        })
    }

    fn accepted_review(id: &str, lane_id: &str, planner: &str, reviewer: &str) -> Value {
        json!({
            "schema_version": "verifier-lane-review/v1",
            "id": id,
            "lane_decision_id": lane_id,
            "requirement_id": "REQ-M",
            "contract_clause": "C-MIG",
            "proof_seed_id": "PS-M",
            "verifier": "verus",
            "reviewer_disposition": "accepted",
            "finding_refs": [],
            "planner_invocation_id": planner,
            "reviewer_invocation_id": reviewer,
            "owner_state": 4,
            "status": "planned",
        })
    }

    fn coverage_codes(obligations: Vec<Value>, lanes: Vec<Value>, reviews: Vec<Value>) -> Vec<String> {
        let mut findings = Vec::new();
        check_risk_profile_coverage(&obligations, &lanes, &reviews, &mut findings);
        findings
            .iter()
            .filter(|f| f.finding_code == "E_LANE_DECISION_MISSING")
            .map(|f| f.message.clone())
            .collect()
    }

    fn equality_proptest_obligation() -> Value {
        json!({
            "requirement_id": "REQ-M",
            "contract_clause": "C-MIG",
            "risk": "equality",
            "verifier": "proptest",
        })
    }

    fn hashed_verus_na() -> Value {
        na_lane(
            "VLD-VERUS",
            "verus",
            vec![Value::String(format!("contract.md clause C-MIG SHA-256:{TEST_HASH}"))],
            "surface_absent",
            "Migration-only delta moves crate::mig::moved_fn byte-identical; \
             equality risk_tags admit no new arithmetic surface for verus here; \
             see contract.md clause C-MIG hash above.",
        )
    }

    #[test]
    fn coverage_honors_accepted_hashed_na() {
        let missing = coverage_codes(
            vec![equality_proptest_obligation()],
            vec![hashed_verus_na()],
            vec![accepted_review("VLR-1", "VLD-VERUS", "planner-inv-1", "reviewer-inv-9")],
        );
        assert!(missing.is_empty(), "accepted hashed N/A must excuse verus; got: {missing:?}");
    }

    #[test]
    fn coverage_flags_missing_lane_decision_without_na() {
        let missing = coverage_codes(vec![equality_proptest_obligation()], vec![], vec![]);
        assert!(missing.iter().any(|m| m.contains("\"verus\"")),
            "no lane decision at all must still flag verus; got: {missing:?}");
    }

    #[test]
    fn coverage_flags_na_without_evidence_refs() {
        let lane = na_lane("VLD-VERUS", "verus", vec![], "surface_absent",
            "Migration-only delta; verus surface absent per contract.");
        let missing = coverage_codes(
            vec![equality_proptest_obligation()],
            vec![lane],
            vec![accepted_review("VLR-1", "VLD-VERUS", "planner-inv-1", "reviewer-inv-9")],
        );
        assert!(missing.iter().any(|m| m.contains("\"verus\"")),
            "N/A without evidence refs must still flag verus; got: {missing:?}");
    }

    #[test]
    fn coverage_flags_na_with_unhashed_refs() {
        let lane = na_lane("VLD-VERUS", "verus",
            vec![Value::String("see contract.md, trust me".into())],
            "surface_absent",
            "Migration-only delta; verus surface absent per contract.");
        let missing = coverage_codes(
            vec![equality_proptest_obligation()],
            vec![lane],
            vec![accepted_review("VLR-1", "VLD-VERUS", "planner-inv-1", "reviewer-inv-9")],
        );
        assert!(missing.iter().any(|m| m.contains("\"verus\"")),
            "N/A with unhashed refs must still flag verus; got: {missing:?}");
    }

    #[test]
    fn coverage_flags_na_with_weak_reason() {
        let lane = na_lane("VLD-VERUS", "verus",
            vec![Value::String(format!("SHA-256:{TEST_HASH}"))],
            "surface_absent",
            "Not needed.");
        let missing = coverage_codes(
            vec![equality_proptest_obligation()],
            vec![lane],
            vec![accepted_review("VLR-1", "VLD-VERUS", "planner-inv-1", "reviewer-inv-9")],
        );
        assert!(missing.iter().any(|m| m.contains("\"verus\"")),
            "N/A with weak reason must still flag verus; got: {missing:?}");
    }

    #[test]
    fn coverage_flags_na_with_wrong_limitation_kind() {
        let lane = na_lane("VLD-VERUS", "verus",
            vec![Value::String(format!("SHA-256:{TEST_HASH}"))],
            "too_hard",
            "Migration-only delta; verus surface absent per contract.");
        let missing = coverage_codes(
            vec![equality_proptest_obligation()],
            vec![lane],
            vec![accepted_review("VLR-1", "VLD-VERUS", "planner-inv-1", "reviewer-inv-9")],
        );
        assert!(missing.iter().any(|m| m.contains("\"verus\"")),
            "N/A with untyped limitation_kind must still flag verus; got: {missing:?}");
    }

    #[test]
    fn coverage_flags_na_without_accepted_review() {
        let missing = coverage_codes(
            vec![equality_proptest_obligation()],
            vec![hashed_verus_na()],
            vec![],
        );
        assert!(missing.iter().any(|m| m.contains("\"verus\"")),
            "hashed N/A without any review must still flag verus; got: {missing:?}");
    }

    #[test]
    fn coverage_flags_na_with_rejected_review() {
        let mut review = accepted_review("VLR-1", "VLD-VERUS", "planner-inv-1", "reviewer-inv-9");
        review["reviewer_disposition"] = Value::String("rejected".into());
        let missing = coverage_codes(
            vec![equality_proptest_obligation()],
            vec![hashed_verus_na()],
            vec![review],
        );
        assert!(missing.iter().any(|m| m.contains("\"verus\"")),
            "N/A with rejected review must still flag verus; got: {missing:?}");
    }

    #[test]
    fn coverage_flags_na_with_self_stamped_review() {
        let missing = coverage_codes(
            vec![equality_proptest_obligation()],
            vec![hashed_verus_na()],
            vec![accepted_review("VLR-1", "VLD-VERUS", "planner-inv-1", "planner-inv-1")],
        );
        assert!(missing.iter().any(|m| m.contains("\"verus\"")),
            "N/A with self-stamped review must still flag verus; got: {missing:?}");
    }

    #[test]
    fn coverage_flags_na_review_for_other_lane() {
        let missing = coverage_codes(
            vec![equality_proptest_obligation()],
            vec![hashed_verus_na()],
            vec![accepted_review("VLR-1", "VLD-OTHER", "planner-inv-1", "reviewer-inv-9")],
        );
        assert!(missing.iter().any(|m| m.contains("\"verus\"")),
            "review for another lane must not excuse verus; got: {missing:?}");
    }

    #[test]
    fn contains_sha256_hex_accepts_plain_and_prefixed_hashes() {
        assert!(contains_sha256_hex(TEST_HASH));
        assert!(contains_sha256_hex(&format!("contract.md SHA-256:{TEST_HASH} trailing")));
        assert!(!contains_sha256_hex(""));
        assert!(!contains_sha256_hex("see contract.md, trust me"));
        assert!(!contains_sha256_hex("0123456789abcdef"));
    }

    #[test]
    fn run_all_checks_passes_migration_delta_with_accepted_na_lanes() {
        let obligation = |id: &str, risk: &str| {
            json!({
                "schema_version": "proof-obligation/v1",
                "id": id,
                "requirement_id": "REQ-M",
                "contract_clause": "C-MIG",
                "domain_claim": "migration-only move preserves behavior",
                "risk": risk,
                "risk_tags": ["migration"],
                "verifier": "proptest",
                "artifact": "tests/migration_parity.rs",
                "target": "crate::mig::moved_fn",
                "command": "PROPTEST_CASES=10000 cargo test --test migration_parity --release",
                "workdir": "/abs",
                "expected_evidence": "test result: ok; anti-invariant invalid_input class rejected",
                "assumptions": [],
                "model_bounds": {"cases": 10000},
                "tool_metadata": {"tool": "proptest", "version_pin": "proptest@1.7"},
                "trusted_base_refs": [],
                "required": true,
                "behavior_affecting": false,
                "mode": "verify-proof",
                "owner_state": 4,
                "rerun_from": 4,
                "status": "planned",
            })
        };
        let na = |id: &str, verifier: &str| {
            na_lane(
                id,
                verifier,
                vec![Value::String(format!("contract.md clause C-MIG SHA-256:{TEST_HASH}"))],
                "surface_absent",
                "Migration-only delta moves crate::mig::moved_fn byte-identical with \
                 source-identity and behavior-parity obligations; this lane has no primary \
                 surface here; see contract.md clause C-MIG hash above.",
            )
        };
        let plan = Plan {
            obligations: vec![
                obligation("PO-SRC", "equality"),
                obligation("PO-PAR", "hostile_input"),
            ],
            lane_decisions: vec![
                json!({
                    "schema_version": "verifier-lane-decision/v1",
                    "id": "VLD-PROPT",
                    "requirement_id": "REQ-M",
                    "contract_clause": "C-MIG",
                    "proof_seed_id": "PS-M",
                    "verifier": "proptest",
                    "risk_tags": ["migration"],
                    "applicability": "required",
                    "decision_reason": "Source-identity and behavior-parity claims need randomized property pressure per risk_tags migration.",
                    "required_obligation_ids": ["PO-SRC", "PO-PAR"],
                    "non_applicability_evidence_refs": [],
                    "limitation_kind": "",
                    "owner_state": 4,
                    "status": "planned",
                }),
                na("VLD-VERUS", "verus"),
                na("VLD-KANI", "kani"),
                na("VLD-FUZZ", "cargo-fuzz"),
            ],
            waivers: vec![],
            lane_reviews: vec![
                accepted_review("VLR-VERUS", "VLD-VERUS", "planner-inv-1", "reviewer-inv-9"),
                accepted_review("VLR-KANI", "VLD-KANI", "planner-inv-1", "reviewer-inv-9"),
                accepted_review("VLR-FUZZ", "VLD-FUZZ", "planner-inv-1", "reviewer-inv-9"),
            ],
        };
        let mut findings = Vec::new();
        run_all_checks(&plan, &Default::default(), &mut findings);
        let codes: Vec<&str> = findings.iter().map(|f| f.finding_code.as_str()).collect();
        assert!(!codes.contains(&"E_LANE_DECISION_MISSING"),
            "migration delta with accepted hashed N/A lanes must not flag coverage; got: {codes:?}");
        assert!(findings.is_empty(), "fully evidenced plan must yield zero findings; got: {codes:?}");
    }

    fn tempdir() -> PathBuf {
        // Use CARGO_MANIFEST_DIR/.tmp-tests/<nano> rather than std::env::temp_dir()
        // because the system /tmp may have inode or quota limits on shared hosts.
        let manifest = std::env::var("CARGO_MANIFEST_DIR")
            .expect("CARGO_MANIFEST_DIR must be set in tests");
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let base = PathBuf::from(manifest)
            .join(".tmp-tests")
            .join(format!("run-{nanos}"));
        std::fs::create_dir_all(&base).unwrap();
        base
    }
}

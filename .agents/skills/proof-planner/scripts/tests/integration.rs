//! Integration tests: invoke the validate-plan and quick-validate binaries
//! against the canonical example plans shipped with the skill.
//!
//! These tests fail if the binaries break their pass/fail contract on the
//! good-plan / bad-plan fixtures.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn workspace_root() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR not set; tests must be run via cargo");
    PathBuf::from(manifest_dir)
        .parent()
        .expect("scripts/ has no parent")
        .to_path_buf()
}

fn example(name: &str) -> PathBuf {
    workspace_root().join("examples").join(name)
}

fn run_validate(plan_dir: &Path) -> Output {
    let exe = std::env::var("CARGO_BIN_EXE_validate-plan")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            workspace_root()
                .join("scripts")
                .join("target")
                .join("release")
                .join("validate-plan")
        });
    Command::new(&exe)
        .arg(plan_dir)
        .output()
        .expect("failed to spawn validate-plan binary")
}

fn run_quick_validate() -> Output {
    let exe = std::env::var("CARGO_BIN_EXE_quick-validate")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            workspace_root()
                .join("scripts")
                .join("target")
                .join("release")
                .join("quick-validate")
        });
    Command::new(&exe)
        .args(["--mirror", &workspace_root().display().to_string()])
        .output()
        .expect("failed to spawn quick-validate binary")
}

#[test]
fn good_plan_passes_with_zero_findings() {
    let out = run_validate(&example("good-plan"));
    assert!(out.status.success(),
        "validate-plan against good-plan must exit 0; got: {:?}\nstderr: {}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("\"findings_total\":0"),
        "stdout must report findings_total: 0; got: {stdout}");
    assert!(stdout.contains("\"status\":\"PASS\""),
        "stdout must report status PASS; got: {stdout}");
}

#[test]
fn bad_plan_fails_with_expected_finding_codes() {
    let out = run_validate(&example("bad-plan"));
    assert!(!out.status.success(),
        "validate-plan against bad-plan must exit non-zero; got: {:?}",
        out.status.code());
    let stdout = String::from_utf8_lossy(&out.stdout);
    for required in [
        "E_BEHAVIOR_WAIVER",
        "E_VERUS_DISCONNECTED_SPEC",
        "E_SOURCE_REF_SHAPE",
        "E_PROOF_SMOKE_MISSING",
        "E_MIRI_PATH_SKIPPED",
        "E_MIRI_TOOLCHAIN_UNPINNED",
        "E_LOOM_NOT_MODELED",
        "E_FUZZ_BUDGET_ABSENT",
        "E_FUZZ_NO_ORACLE",
        "E_PROPTEST_BUDGET_ABSENT",
        "E_PROPTEST_NO_ANTI_INVARIANT",
        "E_RESOURCE_UNGOVERNED",
        "E_LANE_DECISION_WEAK",
        "E_LANE_DECISION_DUPLICATE",
        "E_LANE_OBLIGATION_MISSING",
        "E_TRUST_LEDGER_INCOMPLETE",
        "E_WAIVER_LIFECYCLE_INVALID",
    ] {
        assert!(stdout.contains(required),
            "expected finding code {required} in bad-plan stdout");
    }
}

#[test]
fn bad_plan_finding_count_exceeds_threshold() {
    let out = run_validate(&example("bad-plan"));
    let stdout = String::from_utf8_lossy(&out.stdout);
    // Robust parse: pull every integer out of the summary line and pick the one
    // that follows `findings_total`.
    let summary_line = stdout.lines()
        .rev()
        .find(|l| l.contains("findings_total"))
        .expect("summary line must include findings_total");
    let mut count: Option<usize> = None;
    let rest = summary_line.split("\"findings_total\":").nth(1).unwrap_or("");
    for tok in rest.split(|c: char| !c.is_ascii_digit()) {
        if !tok.is_empty() {
            if let Ok(n) = tok.parse::<usize>() {
                count = Some(n);
                break;
            }
        }
    }
    let count = count.expect("could not parse findings_total from summary");
    assert!(count >= 50,
        "bad-plan must surface many findings; got {count}\nfull stdout: {stdout}");
}

#[test]
fn quick_validate_passes_against_skill_tree() {
    let out = run_quick_validate();
    assert!(out.status.success(),
        "quick-validate must exit 0; got: {:?}\nstderr: {}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("\"status\":\"PASS\""),
        "quick-validate must report PASS; got: {stdout}");
    assert!(stdout.contains("\"references\":16"),
        "must report 16 references; got: {stdout}");
    assert!(stdout.contains("\"templates\":7"),
        "must report 7 templates (TLA+ removed); got: {stdout}");
    assert!(stdout.contains("\"workflows\":5"),
        "must report 5 workflows; got: {stdout}");
}

#[test]
fn bad_plan_emits_no_pass_row_from_planner() {
    let out = run_validate(&example("bad-plan"));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(!stdout.contains("\"status\":\"PASS\""),
        "bad-plan must never yield status: PASS");
    assert!(!stdout.contains("\"disposition\":\"blocker_pass\""),
        "planner must never emit a PASS row");
}

#[test]
fn plan_with_tla_plus_verifier_is_rejected() {
    use std::io::Write;
    let manifest = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR must be set");
    let dir = PathBuf::from(manifest)
        .join(".tmp-tests")
        .join(format!(
            "tla-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
    std::fs::create_dir_all(&dir).unwrap();
    let mut p = std::fs::File::create(dir.join("proof-obligations.planned.jsonl")).unwrap();
    writeln!(p, "{}", serde_json::json!({
        "schema_version": "proof-obligation/v1",
        "id": "PO-TLA",
        "requirement_id": "REQ-TLA",
        "contract_clause": "C-TLA",
        "domain_claim": "TLA+ test",
        "risk": "temporal_liveness",
        "risk_tags": ["temporal_workflow"],
        "verifier": "tla-plus",
        "artifact": "verification/tla/x.tla",
        "target": "crate::w::w",
        "command": "tlc -config x.cfg x.tla",
        "workdir": "/abs",
        "expected_evidence": "Model checking completed",
        "assumptions": [],
        "model_bounds": {},
        "tool_metadata": {"tool": "tlc", "version_pin": "tla2tools.jar@2024"},
        "trusted_base_refs": [],
        "required": true,
        "behavior_affecting": true,
        "mode": "verify-proof",
        "owner_state": 4,
        "rerun_from": 4,
        "status": "planned"
    })).unwrap();
    drop(p);
    let mut p = std::fs::File::create(dir.join("verifier-lane-decisions.jsonl")).unwrap();
    writeln!(p, "{}", serde_json::json!({
        "schema_version": "verifier-lane-decision/v1",
        "id": "VLD-TLA",
        "requirement_id": "REQ-TLA",
        "contract_clause": "C-TLA",
        "proof_seed_id": "PS-TLA",
        "verifier": "tla-plus",
        "risk_tags": ["temporal_workflow"],
        "applicability": "required",
        "decision_reason": "test",
        "required_obligation_ids": ["PO-TLA"],
        "non_applicability_evidence_refs": [],
        "limitation_kind": "",
        "owner_state": 4,
        "status": "planned"
    })).unwrap();
    drop(p);
    std::fs::write(dir.join("waiver-candidates.jsonl"), "").unwrap();
    let out = run_validate(&dir);
    assert!(!out.status.success(),
        "tla-plus must be rejected; got exit {:?}",
        out.status.code());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("tla-plus"),
        "stdout must mention tla-plus as rejected; got: {stdout}");
    let _ = std::fs::remove_dir_all(&dir);
}

//! Proof-planner validator binary (IO shell).
//!
//! Reads a plan directory, runs all gates via the library, emits JSONL
//! findings to stdout and a summary to stderr. No unwrap, no panic, no
//! expect in production code (holzman-rust doctrine).

use std::path::PathBuf;
use std::process::ExitCode;

use proof_planner_validate::{
    check_command_specificity, check_target_shape, check_tool_marker, load_plan,
    run_all_checks, Finding, Plan, Severity, Summary, ALLOWED_VERIFIERS,
};

#[derive(Debug, Default)]
struct CliArgs {
    plan_dir: Option<PathBuf>,
    strict: bool,
}

fn parse_args() -> Result<CliArgs, String> {
    let mut args = CliArgs::default();
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--strict" => args.strict = true,
            "-h" | "--help" => {
                print_help();
                std::process::exit(0);
            }
            other if other.starts_with("--") => {
                return Err(format!("unknown flag: {other}"));
            }
            other => {
                if args.plan_dir.is_some() {
                    return Err(format!("unexpected positional: {other}"));
                }
                args.plan_dir = Some(PathBuf::from(other));
            }
        }
    }
    Ok(args)
}

fn print_help() {
    println!(
        "validate-plan: deterministic gate runner for proof-planner plans.\n\n\
         Usage: validate-plan <plan_dir> [--strict]\n\n\
         The plan directory must contain:\n  \
         - verifier-lane-decisions.jsonl\n  \
         - proof-obligations.planned.jsonl\n  \
         - waiver-candidates.jsonl  (may be empty)\n  \
         - verifier-lane-reviews.jsonl  (may be absent; reviewer acceptance\n  \
           for not_applicable lanes)\n\n\
         Emits JSONL findings to stdout; one summary line to stderr.\n\
         Exit 0 on PASS, 1 on blocker (or major under --strict)."
    );
}

fn main() -> ExitCode {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::from(2);
        }
    };
    let plan_dir = match args.plan_dir {
        Some(d) => d,
        None => {
            eprintln!("error: missing <plan_dir> argument");
            print_help();
            return ExitCode::from(2);
        }
    };
    let plan = match load_plan(&plan_dir) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error: failed to load plan: {e}");
            return ExitCode::from(2);
        }
    };
    let mut findings = Vec::<Finding>::new();
    run_all_checks(&plan, &Default::default(), &mut findings);

    let plan_str = plan_dir.display().to_string();
    let summary = summarize(&plan, &findings, &plan_str);
    let blocker_count = findings.iter().filter(|f| f.severity == Severity::Blocker).count();
    let major_count = findings.iter().filter(|f| f.severity == Severity::Major).count();

    for f in &findings {
        match serde_json::to_string(f) {
            Ok(s) => println!("{s}"),
            Err(e) => {
                eprintln!("error: failed to serialize finding: {e}");
                return ExitCode::from(2);
            }
        }
    }
    match serde_json::to_string(&summary) {
        Ok(s) => println!("{s}"),
        Err(e) => {
            eprintln!("error: failed to serialize summary: {e}");
            return ExitCode::from(2);
        }
    }

    if blocker_count > 0 || (args.strict && major_count > 0) {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

fn summarize(plan: &Plan, findings: &[Finding], plan_dir: &str) -> Summary {
    let mut summary = Summary {
        plan_dir: plan_dir.to_string(),
        status: "PASS",
        ..Summary::default()
    };
    summary.obligations = plan.obligations.iter().filter(|v| v.get("_parse_error").is_none()).count();
    summary.lane_decisions = plan.lane_decisions.iter().filter(|v| v.get("_parse_error").is_none()).count();
    summary.waivers = plan.waivers.iter().filter(|v| v.get("_parse_error").is_none()).count();
    summary.lane_reviews = plan.lane_reviews.iter().filter(|v| v.get("_parse_error").is_none()).count();
    summary.findings_total = findings.len();
    summary.findings_blocker = findings.iter().filter(|f| f.severity == Severity::Blocker).count();
    summary.findings_major = findings.iter().filter(|f| f.severity == Severity::Major).count();
    if summary.findings_blocker > 0 {
        summary.status = "FAIL";
    }
    summary
}

// Compile-time checks that exercise the public surface so dead-code does
// not silently rot this crate.
#[allow(dead_code)]
fn _api_smoke() {
    let _ = check_command_specificity("cargo kani --harness foo");
    let _ = check_target_shape("crate::foo::bar");
    let _ = check_tool_marker("VERIFICATION:- SUCCESSFUL");
    let _ = ALLOWED_VERIFIERS;
    let _ = proof_planner_validate::risk_profile_required("arithmetic_overflow");
}

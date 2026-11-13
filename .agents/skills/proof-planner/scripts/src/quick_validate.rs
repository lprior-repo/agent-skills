//! Quick-validate binary for the proof-planner skill tree.
//!
//! Checks structure, frontmatter, references, templates, workflows, evals,
//! and (optionally) mirror parity. No unwrap, no panic (holzman-rust).
//!
//! Usage: quick-validate [--mirror <path>]

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const EXPECTED_REFS: &[&str] = &[
    "obligation-schema.md",
    "risk-taxonomy.md",
    "lane-decision-guide.md",
    "verifier-trigger-matrix.md",
    "defense-depth-matrix.md",
    "implementation-binding.md",
    "tooling-availability-gate.md",
    "evidence-requirements.md",
    "resource-governance.md",
    "waiver-planning-guide.md",
    "plan-quality-gates.md",
    "anti-patterns.md",
    "handoff-contract.md",
    "index.md",
    "state-machine-integration.md",
    "cross-lane-coverage-matrix.md",
];

const EXPECTED_TEMPLATES: &[&str] = &[
    "verus.json", "kani.json", "flux-rs.json", "loom.json",
    "miri.json", "cargo-fuzz.json", "proptest.json",
];

const EXPECTED_WORKFLOWS: &[&str] = &[
    "pure-invariant.md",
    "parser-hostile-input.md",
    "concurrency-channel.md",
    "unsafe-arena-allocator.md",
    "refinement-offset-newtype.md",
];

#[derive(Debug, Default)]
struct CliArgs {
    mirror: Option<PathBuf>,
}

fn parse_args() -> Result<CliArgs, String> {
    let mut args = CliArgs::default();
    let mut iter = std::env::args().skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--mirror" => {
                let next = iter.next().ok_or("--mirror requires a path")?;
                args.mirror = Some(PathBuf::from(next));
            }
            "-h" | "--help" => {
                print_help();
                std::process::exit(0);
            }
            other => return Err(format!("unknown arg: {other}")),
        }
    }
    Ok(args)
}

fn print_help() {
    println!(
        "quick-validate: structural check for the proof-planner skill tree.\n\n\
         Usage: quick-validate [--mirror <path>]\n\n\
         Checks SKILL.md frontmatter, references/, templates/, workflows/, evals/.json."
    );
}

fn skill_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().map(Path::to_path_buf).unwrap_or_else(|| PathBuf::from("."))
}

fn main() -> ExitCode {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::from(2);
        }
    };
    let dir = skill_dir();
    let mut errors: Vec<String> = Vec::new();

    check_skill_md(&dir, &mut errors);
    check_references(&dir, &mut errors);
    check_templates(&dir, &mut errors);
    check_workflows(&dir, &mut errors);
    check_evals(&dir, &mut errors);
    if let Some(mirror) = &args.mirror {
        check_mirror_parity(&dir, mirror, &mut errors);
    }

    let status = if errors.is_empty() { "PASS" } else { "FAIL" };
    let payload = serde_json::json!({
        "status": status,
        "skill_dir": dir.display().to_string(),
        "references": EXPECTED_REFS.len(),
        "templates": EXPECTED_TEMPLATES.len(),
        "workflows": EXPECTED_WORKFLOWS.len(),
        "errors": errors,
    });
    match serde_json::to_string(&payload) {
        Ok(s) => println!("{s}"),
        Err(e) => {
            eprintln!("error: failed to serialize: {e}");
            return ExitCode::from(2);
        }
    }
    if status == "FAIL" { ExitCode::from(1) } else { ExitCode::SUCCESS }
}

fn check_skill_md(dir: &Path, errors: &mut Vec<String>) {
    let path = dir.join("SKILL.md");
    let text = match fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) => {
            errors.push(format!("SKILL.md missing: {e}"));
            return;
        }
    };
    if !text.starts_with("---\n") {
        errors.push("SKILL.md must start with YAML frontmatter".to_string());
        return;
    }
    let Some(end) = text.find("\n---\n") else {
        errors.push("SKILL.md frontmatter not closed".to_string());
        return;
    };
    let fm = &text[4..end];
    if !fm.contains("name:") {
        errors.push("frontmatter missing 'name'".to_string());
    }
    if !fm.contains("description:") {
        errors.push("frontmatter missing 'description'".to_string());
    }
}

fn check_dir(dir: &Path, expected: &[&str], label: &str, errors: &mut Vec<String>) {
    for name in expected {
        let p = dir.join(name);
        if !p.is_file() {
            errors.push(format!("missing {label}: {name}"));
        }
    }
}

fn check_references(dir: &Path, errors: &mut Vec<String>) {
    check_dir(&dir.join("references"), EXPECTED_REFS, "reference", errors);
}

fn check_templates(dir: &Path, errors: &mut Vec<String>) {
    check_dir(&dir.join("templates"), EXPECTED_TEMPLATES, "template", errors);
}

fn check_workflows(dir: &Path, errors: &mut Vec<String>) {
    check_dir(&dir.join("workflows"), EXPECTED_WORKFLOWS, "workflow", errors);
}

fn check_evals(dir: &Path, errors: &mut Vec<String>) {
    let p = dir.join("evals").join("evals.json");
    let text = match fs::read_to_string(&p) {
        Ok(t) => t,
        Err(e) => {
            errors.push(format!("evals.json missing: {e}"));
            return;
        }
    };
    let parsed: serde_json::Value = match serde_json::from_str(&text) {
        Ok(v) => v,
        Err(e) => {
            errors.push(format!("evals.json parse error: {e}"));
            return;
        }
    };
    let Some(arr) = parsed.as_array() else {
        errors.push("evals.json must be a list".to_string());
        return;
    };
    for (i, ev) in arr.iter().enumerate() {
        for key in ["name", "prompt", "expect"] {
            if ev.get(key).is_none() {
                errors.push(format!("evals[{i}] missing {key:?}"));
            }
        }
    }
}

fn check_mirror_parity(skill_dir: &Path, mirror: &Path, errors: &mut Vec<String>) {
    if !mirror.is_dir() {
        errors.push(format!("mirror path not a directory: {}", mirror.display()));
        return;
    }
    // Top-level directories excluded from parity: examples/ and evals/ may
    // legitimately differ between the canonical tree and a worktree.
    let exclude: BTreeSet<&str> = ["examples", "evals"].iter().copied().collect();
    // Files inside scripts/ that are excluded: build output and test scratch.
    // These are derived state, not source.
    let scripts_exclude_substrings: &[&str] = &[
        "target/",         // cargo build output
        ".tmp-tests/",     // test scratch directory
        ".rustc_info.json", // cargo internal
    ];
    let walk = match walk_files(skill_dir) {
        Ok(w) => w,
        Err(e) => {
            errors.push(format!("mirror walk failed: {e}"));
            return;
        }
    };
    for src in walk {
        let rel = match src.strip_prefix(skill_dir) {
            Ok(r) => r,
            Err(_) => continue,
        };
        let first = rel.components().next().and_then(|c| c.as_os_str().to_str()).unwrap_or("");
        if exclude.contains(first) {
            continue;
        }
        let rel_str = rel.to_string_lossy();
        if first == "scripts" {
            if scripts_exclude_substrings.iter().any(|s| rel_str.contains(s)) {
                continue;
            }
        }
        let dst = mirror.join(rel);
        let Ok(a) = fs::read(&src) else { continue };
        let Ok(b) = fs::read(&dst) else {
            errors.push(format!("mirror missing: {}", rel.display()));
            continue;
        };
        if a != b {
            errors.push(format!("mirror drift: {}", rel.display()));
        }
    }
}

fn walk_files(root: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.is_file() {
                out.push(path);
            }
        }
    }
    Ok(out)
}

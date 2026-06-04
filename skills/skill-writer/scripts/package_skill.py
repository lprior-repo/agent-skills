#!/usr/bin/env python3
"""Package a validated portable skill folder as a .skill archive."""

from __future__ import annotations

import argparse
import fnmatch
import sys
import zipfile
from pathlib import Path

from quick_validate import validate_skill
from utils import parse_skill_md, skill_dir


EXCLUDED_DIRS = {
    ".git",
    ".hg",
    ".svn",
    "__pycache__",
    ".mypy_cache",
    ".pytest_cache",
    ".ruff_cache",
    "node_modules",
    "dist",
    "target",
    ".venv",
    "venv",
    "evals",
    "workspaces",
}
EXCLUDED_FILES = {
    ".DS_Store",
    ".env",
    "feedback.json",
    "benchmark.json",
    "benchmark.md",
    "review.html",
    "description_optimization.json",
    "description_optimization.html",
}
EXCLUDED_PATTERNS = ["*.pyc", "*.pyo", "*.tmp", "*.log", "*.skill", "*-workspace/*"]


def should_exclude(path: Path, root: Path) -> bool:
    rel = path.relative_to(root)
    if any(part in EXCLUDED_DIRS for part in rel.parts):
        return True
    if path.name in EXCLUDED_FILES:
        return True
    rel_text = rel.as_posix()
    return any(fnmatch.fnmatch(rel_text, pattern) for pattern in EXCLUDED_PATTERNS)


def package_skill(skill_path: str | Path, output_dir: str | Path) -> Path:
    errors = validate_skill(skill_path)
    if errors:
        raise ValueError("validation failed: " + "; ".join(errors))

    parsed = parse_skill_md(skill_path)
    root = skill_dir(skill_path)
    output = Path(output_dir).expanduser().resolve()
    output.mkdir(parents=True, exist_ok=True)
    archive = output / f"{parsed['name']}.skill"

    with zipfile.ZipFile(archive, "w", compression=zipfile.ZIP_DEFLATED) as zf:
        for file_path in sorted(root.rglob("*")):
            if file_path.is_dir() or should_exclude(file_path, root):
                continue
            zf.write(file_path, file_path.relative_to(root).as_posix())
    return archive


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("skill_path", help="Path to a skill directory or SKILL.md")
    parser.add_argument(
        "-o",
        "--output-dir",
        default="dist",
        help="Directory for the generated .skill archive",
    )
    args = parser.parse_args()

    try:
        archive = package_skill(args.skill_path, args.output_dir)
    except Exception as exc:
        print(f"ERROR: {exc}", file=sys.stderr)
        return 1

    print(f"Created {archive}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

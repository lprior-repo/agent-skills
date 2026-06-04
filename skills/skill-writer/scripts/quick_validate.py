#!/usr/bin/env python3
"""Validate a portable skill folder."""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

from utils import SkillFormatError, parse_skill_md, skill_md_path


ALLOWED_FRONTMATTER = {
    "name",
    "description",
    "license",
    "compatibility",
    "metadata",
    "allowed-tools",
}


def validate_skill(path: str | Path) -> list[str]:
    errors: list[str] = []
    skill_file = skill_md_path(path)
    try:
        parsed = parse_skill_md(skill_file)
    except SkillFormatError as exc:
        return [str(exc)]

    frontmatter = parsed["frontmatter"]
    unknown = sorted(set(frontmatter) - ALLOWED_FRONTMATTER)
    if unknown:
        errors.append(
            "unsupported portable frontmatter keys: " + ", ".join(unknown)
        )

    name = parsed["name"]
    if not name:
        errors.append("missing required frontmatter field: name")
    elif not re.fullmatch(r"[a-z0-9]+(?:-[a-z0-9]+)*", name):
        errors.append("name must be lowercase kebab-case")

    description = parsed["description"]
    if not description:
        errors.append("missing required frontmatter field: description")
    else:
        if len(description) > 1024:
            errors.append("description must be 1024 characters or fewer")
        if "<" in description or ">" in description:
            errors.append("description must not contain placeholder angle brackets")
        if len(description.split()) < 6:
            errors.append("description is too short to be useful trigger guidance")

    compatibility = frontmatter.get("compatibility")
    if compatibility is not None and len(str(compatibility)) > 500:
        errors.append("compatibility must be 500 characters or fewer")

    if not parsed["body"].strip():
        errors.append("SKILL.md body must not be empty")

    return errors


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "skill_paths",
        nargs="+",
        help="Path(s) to skill directories or SKILL.md files",
    )
    parser.add_argument("--quiet", action="store_true", help="Only print failures")
    args = parser.parse_args()

    failed = False
    for skill_path in args.skill_paths:
        errors = validate_skill(skill_path)
        if errors:
            failed = True
            for error in errors:
                print(f"ERROR {skill_path}: {error}", file=sys.stderr)
            continue
        if not args.quiet:
            parsed = parse_skill_md(skill_md_path(skill_path))
            print(f"OK: {parsed['name']} validated")
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())

#!/usr/bin/env python3
"""Shared helpers for portable skill-writer scripts.

The helpers in this file deliberately avoid any host-specific skill registry,
command directory, or assistant product. Callers pass command templates and
paths explicitly.
"""

from __future__ import annotations

import json
import os
import re
import shlex
import shutil
import subprocess
import time
from pathlib import Path
from typing import Any


FRONTMATTER_RE = re.compile(r"^---\s*\n(.*?)\n---\s*\n?", re.DOTALL)


class SkillFormatError(ValueError):
    """Raised when a skill folder or SKILL.md cannot be parsed."""


def skill_md_path(skill_path: str | Path) -> Path:
    path = Path(skill_path).expanduser().resolve()
    if path.is_dir():
        path = path / "SKILL.md"
    return path


def skill_dir(skill_path: str | Path) -> Path:
    path = skill_md_path(skill_path)
    return path.parent


def read_text(path: str | Path) -> str:
    return Path(path).read_text(encoding="utf-8")


def write_text(path: str | Path, content: str) -> None:
    target = Path(path)
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(content, encoding="utf-8")


def read_json(path: str | Path) -> Any:
    return json.loads(read_text(path))


def write_json(path: str | Path, data: Any) -> None:
    write_text(path, json.dumps(data, indent=2, sort_keys=True) + "\n")


def split_frontmatter(content: str) -> tuple[str, str]:
    match = FRONTMATTER_RE.match(content)
    if not match:
        raise SkillFormatError("SKILL.md must start with YAML frontmatter fenced by ---")
    return match.group(1), content[match.end() :]


def parse_frontmatter(raw: str) -> dict[str, Any]:
    """Parse simple YAML frontmatter with a stdlib fallback.

    PyYAML is used when installed. The fallback intentionally supports the
    simple key/value shape expected for portable skills; complex host-specific
    metadata should be kept out of the portable core anyway.
    """

    try:
        import yaml  # type: ignore

        parsed = yaml.safe_load(raw) or {}
        if not isinstance(parsed, dict):
            raise SkillFormatError("frontmatter must parse to a mapping")
        return dict(parsed)
    except ImportError:
        return _parse_simple_yaml_mapping(raw)


def _parse_simple_yaml_mapping(raw: str) -> dict[str, Any]:
    result: dict[str, Any] = {}
    lines = raw.splitlines()
    index = 0
    while index < len(lines):
        line = lines[index]
        index += 1
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        match = re.match(r"^([A-Za-z0-9_-]+):\s*(.*)$", line)
        if not match:
            raise SkillFormatError(
                "frontmatter uses YAML features that require PyYAML to validate"
            )
        key, value = match.group(1), match.group(2).strip()
        if value in {"|", ">"}:
            block: list[str] = []
            while index < len(lines) and (
                lines[index].startswith(" ") or lines[index].startswith("\t")
            ):
                block.append(lines[index].strip())
                index += 1
            result[key] = "\n".join(block)
        else:
            result[key] = _unquote_scalar(value)
    return result


def _unquote_scalar(value: str) -> str:
    if len(value) >= 2 and value[0] == value[-1] and value[0] in {'"', "'"}:
        return value[1:-1]
    return value


def parse_skill_md(skill_path: str | Path) -> dict[str, Any]:
    path = skill_md_path(skill_path)
    if not path.exists():
        raise SkillFormatError(f"missing SKILL.md at {path}")
    content = read_text(path)
    raw_frontmatter, body = split_frontmatter(content)
    frontmatter = parse_frontmatter(raw_frontmatter)
    return {
        "path": str(path),
        "dir": str(path.parent),
        "frontmatter": frontmatter,
        "name": str(frontmatter.get("name", "")),
        "description": str(frontmatter.get("description", "")),
        "body": body,
        "content": content,
        "raw_frontmatter": raw_frontmatter,
    }


def yaml_quote(value: str) -> str:
    return json.dumps(value, ensure_ascii=True)


def update_frontmatter_field(skill_path: str | Path, field: str, value: str) -> None:
    path = skill_md_path(skill_path)
    content = read_text(path)
    raw_frontmatter, body = split_frontmatter(content)
    lines = raw_frontmatter.splitlines()
    rendered = f"{field}: {yaml_quote(value)}"
    replaced = False
    for index, line in enumerate(lines):
        if re.match(rf"^{re.escape(field)}\s*:", line):
            lines[index] = rendered
            replaced = True
            break
    if not replaced:
        lines.append(rendered)
    write_text(path, "---\n" + "\n".join(lines) + "\n---\n" + body)


def safe_slug(value: str, fallback: str = "item") -> str:
    slug = re.sub(r"[^a-z0-9]+", "-", value.lower()).strip("-")
    return slug[:80] or fallback


def normalize_eval_items(data: dict[str, Any]) -> list[dict[str, Any]]:
    raw_items = data.get("evals", data.get("queries", []))
    if not isinstance(raw_items, list):
        raise SkillFormatError("eval file must contain an 'evals' or 'queries' list")
    items: list[dict[str, Any]] = []
    for index, raw in enumerate(raw_items, start=1):
        if not isinstance(raw, dict):
            raise SkillFormatError(f"eval item {index} must be an object")
        prompt = raw.get("prompt", raw.get("query", ""))
        if not isinstance(prompt, str) or not prompt.strip():
            raise SkillFormatError(f"eval item {index} must contain prompt or query")
        item = dict(raw)
        item.setdefault("id", index)
        item.setdefault("name", safe_slug(prompt, f"eval-{index}"))
        item["prompt"] = prompt
        if "expectations" not in item:
            expected = item.get("expected_output")
            item["expectations"] = [expected] if isinstance(expected, str) and expected else []
        items.append(item)
    return items


def load_eval_file(path: str | Path) -> tuple[dict[str, Any], list[dict[str, Any]]]:
    data = read_json(path)
    if not isinstance(data, dict):
        raise SkillFormatError("eval file must be a JSON object")
    return data, normalize_eval_items(data)


class _SafeFormatMap(dict[str, str]):
    def __missing__(self, key: str) -> str:
        return ""


def render_command_template(template: str, values: dict[str, Any]) -> str:
    quoted = _SafeFormatMap({key: shlex.quote(str(value)) for key, value in values.items()})
    return template.format_map(quoted)


def run_command_template(
    template: str,
    values: dict[str, Any],
    *,
    stdin: str = "",
    cwd: str | Path | None = None,
    timeout: int | None = None,
    env: dict[str, str] | None = None,
) -> dict[str, Any]:
    command = render_command_template(template, values)
    merged_env = os.environ.copy()
    if env:
        merged_env.update(env)
    started = time.monotonic()
    try:
        proc = subprocess.run(
            command,
            input=stdin,
            text=True,
            shell=True,
            cwd=str(cwd) if cwd else None,
            env=merged_env,
            timeout=timeout,
            capture_output=True,
            check=False,
        )
        timed_out = False
    except subprocess.TimeoutExpired as exc:
        return {
            "command": command,
            "exit_code": None,
            "stdout": exc.stdout or "",
            "stderr": exc.stderr or f"command timed out after {timeout} seconds",
            "duration_seconds": round(time.monotonic() - started, 3),
            "timed_out": True,
        }
    return {
        "command": command,
        "exit_code": proc.returncode,
        "stdout": proc.stdout,
        "stderr": proc.stderr,
        "duration_seconds": round(time.monotonic() - started, 3),
        "timed_out": timed_out,
    }


def transcript_from_result(result: dict[str, Any]) -> str:
    return "\n".join(
        [
            f"Command: {result.get('command', '')}",
            f"Exit code: {result.get('exit_code')}",
            f"Duration seconds: {result.get('duration_seconds')}",
            "",
            "## stdout",
            str(result.get("stdout", "")),
            "",
            "## stderr",
            str(result.get("stderr", "")),
            "",
        ]
    )


def copy_skill_tree(source: str | Path, dest: str | Path) -> None:
    src = skill_dir(source)
    dst = Path(dest)
    resolved_src = src.resolve()
    resolved_dst = dst.resolve()
    if resolved_dst == resolved_src or resolved_src in resolved_dst.parents:
        raise SkillFormatError("copy destination must not be inside the source skill tree")
    if dst.exists():
        shutil.rmtree(dst)
    ignore = shutil.ignore_patterns(
        "__pycache__",
        ".git",
        ".pytest_cache",
        ".ruff_cache",
        "*.pyc",
        "*.skill",
        "*-workspace",
    )
    shutil.copytree(src, dst, ignore=ignore)

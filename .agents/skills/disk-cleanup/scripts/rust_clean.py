#!/usr/bin/env python3
"""Reap stale Cargo targets system-wide while protecting personal/model data."""

from __future__ import annotations

import argparse
import os
import shutil
import subprocess
import sys
import time
from pathlib import Path


HOME = Path.home()
PROTECTED_PATHS = {
    (HOME / "Documents").resolve(),
    (HOME / "Downloads").resolve(),
    (HOME / ".cache/r2-kernel-baseline-Os9ai0").resolve(),
    (HOME / ".cache/athletic-agent-worktrees").resolve(),
    (HOME / ".local/share/athletic-rust-pipeline").resolve(),
    (HOME / "ninfer-turbofusion/out").resolve(),
    Path("/var/cache/pacman/pkg").resolve(),
    Path("/var/lib").resolve(),
    Path("/var/log").resolve(),
    Path("/var/spool").resolve(),
    Path("/var/db").resolve(),
}
PROTECTED_DIR_NAMES = {
    "downloads",
    "documents",
    "steam",
    ".steam",
    "steamapps",
    "models",
    "model-store",
    "model_store",
    ".ollama",
    "ollama",
    "huggingface",
    "transformers",
    "llama.cpp",
    "fuzz",
    "evidence",
    "receipts",
    "backups",
    "snapshots",
    ".snapshots",
}
PSEUDO_OR_EXTERNAL_DIR_NAMES = {"proc", "sys", "dev", "run", "mnt", "media"}
REAPER_DEFAULT = HOME / ".local/bin/target-reaper"
REPORT_PATHS = (HOME / "Documents", HOME / "Downloads")
THREE_GIB = 3 * 1024**3
CACHE_ROOT = HOME / ".cache"
PROTECTED_CACHE_PREFIXES = (
    "athletic-",
    "split-",
    "q01l-",
    "r14",
)
PROTECTED_CACHE_NAMES = {
    "cen-native-vm",
    "titania-check-tests",
    "vb-cleanup-30r-audit",
    "test-quota",
}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--root",
        type=Path,
        default=Path("/"),
        help="tree to scan (default: the local system tree; protected paths remain excluded)",
    )
    parser.add_argument("--age-days", type=int, default=1)
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument(
        "--skip-protected-report",
        action="store_true",
        help="do not print the report-only Documents/Downloads inventory",
    )
    parser.add_argument("--reaper", type=Path, default=REAPER_DEFAULT)
    return parser.parse_args()


def is_within(path: Path, parent: Path) -> bool:
    try:
        path.relative_to(parent)
        return True
    except ValueError:
        return False


def is_protected(path: Path) -> bool:
    return any(is_within(path, protected) for protected in PROTECTED_PATHS)


def scan_roots(root: Path) -> list[Path]:
    if root != Path("/"):
        return [root]
    findmnt = shutil.which("findmnt")
    if not findmnt:
        print("findmnt not found; only scanning / (mounted subvolumes may be skipped)", file=sys.stderr)
        return [root]

    root_mount = subprocess.run(
        [findmnt, "-n", "-T", str(root), "-o", "SOURCE,FSTYPE"],
        check=False,
        capture_output=True,
        text=True,
    )
    fields = root_mount.stdout.split()
    if root_mount.returncode != 0 or len(fields) != 2 or fields[1] != "btrfs":
        print("root is not Btrfs; scanning only /", file=sys.stderr)
        return [root]
    source_device = fields[0].split("[", 1)[0]

    mounts = subprocess.run(
        [findmnt, "-rn", "-t", "btrfs", "-o", "TARGET,SOURCE"],
        check=False,
        capture_output=True,
        text=True,
    )
    roots = {root}
    for line in mounts.stdout.splitlines():
        mount_fields = line.split()
        if len(mount_fields) != 2:
            continue
        mountpoint, source = mount_fields
        if source.split("[", 1)[0] != source_device:
            continue
        candidate = Path(mountpoint)
        if not is_protected(candidate):
            roots.add(candidate)
    return sorted(roots, key=str)


def target_roots(root: Path) -> list[Path]:
    if is_protected(root):
        return []
    find = shutil.which("find")
    if not find:
        raise RuntimeError("find is required for a system-wide directory scan")

    prune_terms: list[str] = []

    def add_or(term: list[str]) -> None:
        if prune_terms:
            prune_terms.append("-o")
        prune_terms.extend(term)

    for protected in PROTECTED_PATHS:
        add_or(["-path", str(protected)])
    for name in sorted(PROTECTED_DIR_NAMES | PSEUDO_OR_EXTERNAL_DIR_NAMES):
        add_or(["-iname", name])
    # Avoid project/runtime state directories named `var`, while allowing the
    # system /var tree so Cargo targets under /var/cache remain discoverable.
    add_or(["-name", "var", "!", "-path", "/var"])

    command = [
        find,
        str(root),
        "-xdev",
        "(",
        *prune_terms,
        ")",
        "-prune",
        "-o",
        "-type",
        "d",
        "-name",
        "target",
        "-print0",
    ]
    result = subprocess.run(command, check=False, capture_output=True)
    if result.returncode > 1:
        detail = result.stderr.decode(errors="replace").strip()
        if detail:
            print(detail, file=sys.stderr)
        raise RuntimeError(f"find exited with status {result.returncode}")
    if result.returncode == 1:
        errors = result.stderr.decode(errors="replace").splitlines()
        print(
            f"find skipped {len(errors)} inaccessible paths; scanning accessible paths",
            file=sys.stderr,
        )
    return [
        Path(os.fsdecode(item))
        for item in result.stdout.split(b"\0")
        if item
    ]


def report_large_protected_items() -> None:
    print("\nProtected Documents/Downloads items over 3 GiB (report only):")
    for root in REPORT_PATHS:
        if not root.is_dir():
            continue
        print(f"\n{root} — directories through depth 4:")
        du = shutil.which("du")
        if du:
            result = subprocess.run(
                [du, "-x", "-h", "--max-depth=4", "--threshold=3G", str(root)],
                check=False,
                capture_output=True,
                text=True,
            )
            if result.stdout.strip():
                print(result.stdout.rstrip())
            if result.stderr.strip():
                print(result.stderr.rstrip(), file=sys.stderr)
        find = shutil.which("find")
        if find:
            print(f"{root} — individual files over 3 GiB:")
            result = subprocess.run(
                [
                    find,
                    str(root),
                    "-xdev",
                    "-type",
                    "f",
                    "-size",
                    f"+{THREE_GIB}c",
                    "-printf",
                    "%s\t%p\n",
                ],
                check=False,
                capture_output=True,
                text=True,
            )
            if result.stdout.strip():
                print(result.stdout.rstrip())
            if result.stderr.strip():
                print(result.stderr.rstrip(), file=sys.stderr)


def reap(roots: list[Path], args: argparse.Namespace) -> bool:
    print(f"Found {len(roots)} Cargo target directories under {args.root}")
    if not roots:
        return True

    failed = False
    for start in range(0, len(roots), 100):
        command = [str(args.reaper), "--age-days", str(args.age_days)]
        if not args.dry_run:
            command.append("--execute")
        for target in roots[start : start + 100]:
            command.extend(("--root", str(target)))
        result = subprocess.run(command, check=False)
        failed = failed or result.returncode != 0
    return not failed


def cache_candidates(age_days: int) -> list[Path]:
    """Return whole ~/.cache entries with no recent reads or writes."""
    if not CACHE_ROOT.is_dir():
        return []
    cutoff = time.time() - age_days * 24 * 60 * 60
    candidates: list[Path] = []
    for entry in sorted(CACHE_ROOT.iterdir()):
        try:
            if (
                entry.is_symlink()
                or not entry.is_dir()
                or entry.name.lower() in PROTECTED_DIR_NAMES
                or entry.name.lower() in PROTECTED_CACHE_NAMES
                or entry.name.lower().startswith(PROTECTED_CACHE_PREFIXES)
                or is_protected(entry.resolve())
            ):
                continue
            if entry.stat().st_mtime >= cutoff:
                continue
            # Keep a cache if any nested file or directory changed recently.
            fresh = False
            def mark_inaccessible(_error: OSError) -> None:
                nonlocal fresh
                fresh = True

            for current, dirs, files in os.walk(
                entry, followlinks=False, onerror=mark_inaccessible
            ):
                current_path = Path(current)
                for name in dirs + files:
                    child = current_path / name
                    try:
                        if child.is_symlink():
                            continue
                        stat = child.stat(follow_symlinks=False)
                    except OSError:
                        fresh = True
                        break
                    if stat.st_mtime >= cutoff:
                        fresh = True
                        break
                if fresh:
                    break
            if fresh:
                continue
            if process_uses_path(entry):
                print(f"[CACHE SKIP] in use by a process: {entry}")
                continue
            candidates.append(entry)
        except OSError as error:
            print(f"[CACHE SKIP] cannot inspect {entry}: {error}")
    return candidates


def process_uses_path(path: Path) -> bool:
    """Check process file descriptors and working directories without lsof's recursive walk."""
    proc = Path("/proc")
    if not proc.is_dir():
        return True
    target = str(path.resolve())
    try:
        processes = list(proc.iterdir())
    except OSError:
        return True
    for process in processes:
        if not process.name.isdigit():
            continue
        locations = [process / "cwd"]
        fd_dir = process / "fd"
        try:
            locations.extend(fd_dir.iterdir())
        except OSError:
            continue
        for location in locations:
            try:
                opened = os.readlink(location).removesuffix(" (deleted)")
            except OSError:
                continue
            if opened == target or opened.startswith(target + os.sep):
                return True
    return False


def clean_cache_dirs(dry_run: bool, age_days: int) -> bool:
    candidates = cache_candidates(age_days)
    print(f"Found {len(candidates)} stale cache directories under {CACHE_ROOT}")
    success = True
    du = shutil.which("du")
    for path in candidates:
        size = "size unknown"
        if du:
            measured = subprocess.run(
                [du, "-sh", str(path)], check=False, capture_output=True, text=True
            )
            if measured.returncode == 0 and measured.stdout.strip():
                size = measured.stdout.split(maxsplit=1)[0]
        if dry_run:
            print(f"[DRY RUN] {size} {path}")
            continue
        try:
            shutil.rmtree(path)
            print(f"[REMOVED] {size} {path}")
        except OSError as error:
            print(f"[CACHE ERROR] {path}: {error}", file=sys.stderr)
            success = False
    return success


def main() -> int:
    args = parse_args()
    if args.age_days < 1:
        print("refusing age cutoffs below 1 day", file=sys.stderr)
        return 2
    if not args.root.is_dir():
        print(f"root is not a directory: {args.root}", file=sys.stderr)
        return 2
    if not args.reaper.is_file():
        print(f"target-reaper not found: {args.reaper}", file=sys.stderr)
        return 2

    if not args.skip_protected_report:
        report_large_protected_items()
    scan_points = scan_roots(args.root.resolve())
    print("Scanning local filesystem roots: " + ", ".join(map(str, scan_points)))
    roots = sorted(
        {target for point in scan_points for target in target_roots(point)}, key=str
    )
    success = reap(roots, args)
    success = clean_cache_dirs(args.dry_run, args.age_days) and success
    return 0 if success else 1


if __name__ == "__main__":
    raise SystemExit(main())

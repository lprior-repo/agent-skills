#!/usr/bin/env python3
"""Deterministic stdlib-only tests for the Muse bridge subprocess boundary."""

import json
import os
import signal
import subprocess
import sys
import tempfile
import time
import unittest
from pathlib import Path

BRIDGE = Path(__file__).resolve().parents[1] / "scripts/muse-bridge"


def fake_cli(directory: Path, behavior: str) -> Path:
    binary_dir = directory / "bin"
    binary_dir.mkdir()
    fake = binary_dir / "muse"
    body = {
        "success": """
print('{"type":"assistant","text":"mock-ok"}')
print("mock diagnostic", file=sys.stderr)
""",
        "failure": """
print("mock failure", file=sys.stderr)
raise SystemExit(17)
""",
        "overflow": """
sys.stdout.write("X" * 8192)
sys.stdout.flush()
time.sleep(30)
""",
        "sleep": """
prompt = Path(args[args.index("--prompt-file") + 1])
child = subprocess.Popen([sys.executable, "-c", "import time; time.sleep(60)"])
prompt.with_suffix(".child").write_text(str(child.pid))
time.sleep(60)
""",
    }[behavior]
    script = '''#!/usr/bin/env python3
import os
from pathlib import Path
import subprocess
import sys
import time
args = sys.argv[1:]
if args[:3] == ["model-profile", "show", "muse-spark-1.3"]:
    print("model:  muse-spark-1.3")
    raise SystemExit(0)
if not args or args[0] != "exec":
    raise SystemExit(90)
if "META_API_KEY" in os.environ or os.environ.get("MUSE_NO_AUTO_UPDATE") != "1":
    raise SystemExit(91)
pairs = [
    ("--model", "muse-spark-1.3"),
    ("--contributor-mode", "on"),
    ("--reasoning-effort", "low"),
    ("--worktree", "off"),
    ("--approval-mode", "on-request"),
    ("--sandbox-network", "restricted"),
]
for left, right in pairs:
    if left not in args or args[args.index(left) + 1] != right:
        raise SystemExit(92)
for flag in ("--json", "--disable-write", "--disable-shell", "--no-session-log"):
    if flag not in args:
        raise SystemExit(93)
workspace = Path(args[args.index("--workspace") + 1])
prompt = Path(args[args.index("--prompt-file") + 1])
if not workspace.is_dir() or not Path(os.environ.get("TMPDIR", "")).is_dir():
    raise SystemExit(95)
if prompt.read_bytes() != b"Reply exactly: mock-ok":
    raise SystemExit(94)
''' + body
    fake.write_text(script)
    fake.chmod(0o755)
    return binary_dir


class MuseBridgeTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.workspace = self.root / "workspace"
        self.workspace.mkdir()
        self.prompt = self.root / "prompt.txt"
        self.prompt.write_bytes(b"Reply exactly: mock-ok")
        self.mock_tmp = self.root / "mock-system-tmp"
        self.mock_tmp.mkdir()

    def tearDown(self) -> None:
        self.temp.cleanup()

    def command(self, output: Path, timeout: int = 3, output_limit: int = 4096) -> list[str]:
        return [
            str(BRIDGE),
            "--workspace", str(self.workspace),
            "--prompt-file", str(self.prompt),
            "--output-dir", str(output),
            "--timeout-seconds", str(timeout),
            "--max-output-bytes", str(output_limit),
        ]

    def env(self, binary_dir: Path) -> dict[str, str]:
        result = os.environ.copy()
        result["PATH"] = str(binary_dir) + os.pathsep + result.get("PATH", "")
        result["META_API_KEY"] = "mock-only-do-not-print"
        result["TMPDIR"] = str(self.mock_tmp)
        return result

    def run_bridge(self, behavior: str, output: Path, timeout: int = 3,
                   output_limit: int = 4096) -> subprocess.CompletedProcess[bytes]:
        binary_dir = fake_cli(self.root, behavior)
        return subprocess.run(
            self.command(output, timeout, output_limit),
            env=self.env(binary_dir),
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            timeout=8,
            check=False,
        )

    def status(self, output: Path) -> dict[str, object]:
        return json.loads((output / "status.json").read_text())

    def test_success_keeps_outputs_separate_and_permissions_private(self) -> None:
        output = self.root / "success"
        result = self.run_bridge("success", output)
        self.assertEqual(result.returncode, 0, result.stderr)
        status = self.status(output)
        self.assertEqual(status["status"], "succeeded")
        self.assertEqual(status["muse_exit_code"], 0)
        self.assertIn(b"mock-ok", (output / "stdout.jsonl").read_bytes())
        self.assertIn(b"mock diagnostic", (output / "stderr.log").read_bytes())
        self.assertIn(b"muse-spark-1.3", (output / "model-profile.stdout").read_bytes())
        self.assertFalse((output / ".prompt").exists())
        self.assertFalse((output / ".runtime-tmp").exists())
        self.assertEqual(output.stat().st_mode & 0o777, 0o700)
        self.assertEqual((output / "stdout.jsonl").stat().st_mode & 0o777, 0o600)
        self.assertNotIn(b"mock-only-do-not-print", result.stdout + result.stderr)

    def test_workspace_inside_system_temp_is_rejected_before_cli(self) -> None:
        output = self.root / "unsafe-temp-workspace-run"
        workspace = self.mock_tmp / "workspace"
        workspace.mkdir()
        binary_dir = fake_cli(self.root, "success")
        result = subprocess.run(
            [
                str(BRIDGE),
                "--workspace", str(workspace),
                "--prompt-file", str(self.prompt),
                "--output-dir", str(output),
            ],
            env=self.env(binary_dir),
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            timeout=5,
            check=False,
        )
        self.assertEqual(result.returncode, 2)
        self.assertIn(b"outside TMPDIR", result.stderr)
        self.assertFalse(output.exists())

    def test_muse_failure_is_reported_and_captured(self) -> None:
        output = self.root / "failure"
        result = self.run_bridge("failure", output)
        self.assertEqual(result.returncode, 3)
        status = self.status(output)
        self.assertEqual(status["status"], "muse-failed")
        self.assertEqual(status["muse_exit_code"], 17)
        self.assertIn(b"mock failure", (output / "stderr.log").read_bytes())

    def wait_child_pid(self, path: Path, process: subprocess.Popen[bytes]) -> int:
        deadline = time.monotonic() + 4
        while time.monotonic() < deadline:
            if path.exists():
                return int(path.read_text())
            if process.poll() is not None:
                break
            time.sleep(0.02)
        self.fail("mock Muse did not start its child process")
        raise AssertionError("unreachable")

    def assert_gone(self, pid: int) -> None:
        deadline = time.monotonic() + 4
        while time.monotonic() < deadline:
            try:
                os.kill(pid, 0)
            except ProcessLookupError:
                return
            try:
                state = Path(f"/proc/{pid}/stat").read_text().split()[2]
            except (FileNotFoundError, IndexError):
                return
            if state == "Z":
                return
            time.sleep(0.03)
        self.fail(f"owned child process {pid} remained alive")

    def test_timeout_kills_owned_process_group_and_caps_output(self) -> None:
        output = self.root / "timeout"
        result = self.run_bridge("sleep", output, timeout=1, output_limit=256)
        self.assertEqual(result.returncode, 124)
        self.assertEqual(self.status(output)["status"], "timeout")
        pid_file = output / ".prompt.child"
        self.assertTrue(pid_file.exists())
        self.assert_gone(int(pid_file.read_text()))

    def test_sigterm_cancellation_kills_owned_process_group(self) -> None:
        output = self.root / "cancel"
        binary_dir = fake_cli(self.root, "sleep")
        process = subprocess.Popen(
            self.command(output, timeout=30),
            env=self.env(binary_dir),
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        child_pid = self.wait_child_pid(output / ".prompt.child", process)
        process.send_signal(signal.SIGTERM)
        _stdout, stderr = process.communicate(timeout=6)
        self.assertEqual(process.returncode, 130, stderr)
        self.assertEqual(self.status(output)["status"], "cancelled")
        self.assert_gone(child_pid)

    def test_oversized_output_is_capped_and_fails_closed(self) -> None:
        output = self.root / "overflow"
        result = self.run_bridge("overflow", output, output_limit=128)
        self.assertEqual(result.returncode, 125)
        self.assertEqual(self.status(output)["status"], "output-limit")
        self.assertEqual((output / "stdout.jsonl").stat().st_size, 128)


if __name__ == "__main__":
    unittest.main()

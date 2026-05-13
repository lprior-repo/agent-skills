#!/usr/bin/env python3
"""Generate a portable eval review page and optional feedback server."""

from __future__ import annotations

import argparse
import base64
import json
import mimetypes
import sys
import webbrowser
from http.server import BaseHTTPRequestHandler, HTTPServer
from pathlib import Path
from typing import Any


TEXT_SUFFIXES = {".txt", ".md", ".json", ".yaml", ".yml", ".csv", ".log", ".html", ".css", ".js", ".py", ".rs"}


def read_json(path: Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def find_metadata(run_dir: Path, stop: Path) -> dict[str, Any]:
    for parent in [run_dir, *run_dir.parents]:
        if parent == stop.parent:
            break
        candidate = parent / "eval_metadata.json"
        if candidate.exists():
            try:
                return read_json(candidate)
            except Exception:
                return {}
    return {}


def file_payload(path: Path, display_name: str | None = None) -> dict[str, Any]:
    name = display_name or path.name
    suffix = path.suffix.lower()
    mime = mimetypes.guess_type(path.name)[0] or "application/octet-stream"
    if suffix in TEXT_SUFFIXES or mime.startswith("text/"):
        try:
            text = path.read_text(encoding="utf-8")
        except UnicodeDecodeError:
            text = None
        if text is not None:
            return {"name": name, "type": "text", "content": text}
    raw = path.read_bytes()
    b64 = base64.b64encode(raw).decode("ascii")
    if mime.startswith("image/"):
        return {"name": name, "type": "image", "data_uri": f"data:{mime};base64,{b64}"}
    if mime == "application/pdf":
        return {"name": name, "type": "pdf", "data_uri": f"data:{mime};base64,{b64}"}
    return {"name": name, "type": "binary", "data_b64": b64, "mime": mime}


def collect_run(run_dir: Path, workspace: Path) -> dict[str, Any]:
    metadata = find_metadata(run_dir, workspace)
    outputs: list[dict[str, Any]] = []
    transcript = run_dir / "transcript.md"
    if transcript.exists():
        outputs.append(file_payload(transcript, "transcript.md"))
    outputs_dir = run_dir / "outputs"
    if outputs_dir.exists():
        for path in sorted(outputs_dir.rglob("*")):
            if path.is_file():
                outputs.append(file_payload(path, str(path.relative_to(outputs_dir))))
    grading_path = run_dir / "grading.json"
    grading = read_json(grading_path) if grading_path.exists() else None
    rel_id = run_dir.relative_to(workspace).as_posix()
    return {
        "id": rel_id,
        "prompt": metadata.get("prompt", ""),
        "eval_id": metadata.get("eval_id"),
        "eval_name": metadata.get("eval_name"),
        "outputs": outputs,
        "grading": grading,
    }


def collect_data(workspace: Path, skill_name: str = "") -> dict[str, Any]:
    if not workspace.exists():
        raise FileNotFoundError(f"workspace not found: {workspace}")
    run_dirs = sorted({path.parent for path in workspace.rglob("transcript.md")})
    runs = [collect_run(run_dir, workspace) for run_dir in run_dirs]
    benchmark_path = workspace / "benchmark.json"
    benchmark = read_json(benchmark_path) if benchmark_path.exists() else None
    return {"skill_name": skill_name or workspace.name, "runs": runs, "benchmark": benchmark}


def render_viewer(data: dict[str, Any], template_path: Path) -> str:
    template = template_path.read_text(encoding="utf-8")
    embedded = "const EMBEDDED_DATA = " + json.dumps(data) + ";"
    return template.replace("/*__EMBEDDED_DATA__*/", embedded)


class FeedbackHandler(BaseHTTPRequestHandler):
    html_path: Path
    feedback_path: Path

    def _send_json(self, payload: dict[str, Any]) -> None:
        body = json.dumps(payload).encode("utf-8")
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def _send_error_json(self, status: int, message: str) -> None:
        body = json.dumps({"ok": False, "error": message}).encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def do_GET(self) -> None:  # noqa: N802
        if self.path == "/api/feedback":
            if self.feedback_path.exists():
                self._send_json(read_json(self.feedback_path))
            else:
                self._send_json({"reviews": [], "status": "new"})
            return
        body = self.html_path.read_bytes()
        self.send_response(200)
        self.send_header("Content-Type", "text/html; charset=utf-8")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def do_POST(self) -> None:  # noqa: N802
        if self.path != "/api/feedback":
            self.send_error(404)
            return
        try:
            length = int(self.headers.get("Content-Length", "0"))
            body = self.rfile.read(length)
            payload = json.loads(body.decode("utf-8")) if body else {}
        except (ValueError, json.JSONDecodeError):
            self._send_error_json(400, "request body must be valid JSON")
            return
        if not isinstance(payload, dict):
            self._send_error_json(400, "feedback payload must be a JSON object")
            return
        self.feedback_path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
        self._send_json({"ok": True})

    def log_message(self, fmt: str, *args: object) -> None:
        return


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("workspace", help="Eval workspace containing transcript.md files")
    parser.add_argument("--skill-name", default="", help="Display name for the review page")
    parser.add_argument("--output", default="review.html", help="Generated HTML path")
    parser.add_argument("--static", action="store_true", help="Only write HTML; do not start a server")
    parser.add_argument("--port", type=int, default=8765)
    parser.add_argument("--open", action="store_true", help="Open the local review page in a browser")
    args = parser.parse_args()

    try:
        workspace = Path(args.workspace).expanduser().resolve()
        template = Path(__file__).with_name("viewer.html")
        output = Path(args.output).expanduser().resolve()
        output.write_text(render_viewer(collect_data(workspace, args.skill_name), template), encoding="utf-8")
    except Exception as exc:
        print(f"ERROR: {exc}", file=sys.stderr)
        return 1

    print(f"Wrote {output}")
    if args.static:
        return 0

    FeedbackHandler.html_path = output
    FeedbackHandler.feedback_path = workspace / "feedback.json"
    server = HTTPServer(("127.0.0.1", args.port), FeedbackHandler)
    url = f"http://127.0.0.1:{args.port}/"
    print(f"Serving review at {url}")
    print("Press Ctrl-C to stop.")
    if args.open:
        webbrowser.open(url)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("Stopped.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

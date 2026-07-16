"""Deploy and probe the packaged CyberCity Collector binary."""

from __future__ import annotations

import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[1]
DEPLOY_ROOT = ROOT / ".deploy"
READINESS_MARKERS = (
    "loading config from",
    "collector starting: collector-test / cybercity-test",
    "transport initialized",
    "127.0.0.1:9092",
    "cc.events.cybercity-test",
    "cc.commands.cybercity-test",
    "lifecycle transition: initializing -> active",
    "telemetry collector started",
)
FATAL_MARKERS = ("collector failed:", "panicked at", "thread 'main' panicked")


def artifact_binary(artifact_dir: Path) -> Path:
    name = "cybercity-collector.exe" if os.name == "nt" else "cybercity-collector"
    candidate = artifact_dir / "target" / "release" / name
    if not candidate.is_file():
        raise RuntimeError(f"packaged binary is missing: {candidate}")
    return candidate


def toml_literal(value: Path) -> str:
    return "'" + str(value).replace("'", "''") + "'"


def write_probe_config() -> Path:
    probe_input = (DEPLOY_ROOT / "probe-input.log").resolve()
    probe_input.write_text("collector-deploy-probe\n", encoding="utf-8")
    config = DEPLOY_ROOT / "config.toml"
    path = toml_literal(probe_input)
    config.write_text(
        "\n".join(
            (
                'node_id = "collector-test"',
                'service_id = "cybercity-test"',
                'segment = "test"',
                'kafka_broker = "127.0.0.1:9092"',
                f"spool_path = {toml_literal((DEPLOY_ROOT / 'spool').resolve())}",
                "[telemetry]",
                f"log_paths = [{path}]",
                "poll_interval_secs = 1",
                "buffer_size = 16",
                "[policy]",
                "allow_telemetry = true",
                'allowed_command_kinds = ["status", "read_file"]',
                "[[policy.host_permissions]]",
                'kind = "read_file"',
                f"paths = [{path}]",
                "",
            )
        ),
        encoding="utf-8",
    )
    return config


def start(artifact_dir: Path, commit: str) -> None:
    DEPLOY_ROOT.mkdir(exist_ok=True)
    source = artifact_binary(artifact_dir)
    target = DEPLOY_ROOT / source.name
    shutil.copy2(source, target)
    target.chmod(0o755)
    config = write_probe_config()
    environment = os.environ.copy()
    environment["RUST_LOG"] = "info"
    log_path = DEPLOY_ROOT / "collector.log"
    log = log_path.open("wb")
    process = subprocess.Popen(
        [str(target), str(config)],
        cwd=ROOT,
        env=environment,
        stdin=subprocess.DEVNULL,
        stdout=log,
        stderr=subprocess.STDOUT,
        start_new_session=True,
    )
    (DEPLOY_ROOT / "deployment.json").write_text(
        json.dumps({"commit": commit, "pid": process.pid, "log": str(log_path)}) + "\n",
        encoding="utf-8",
    )


def deployment(commit: str) -> tuple[int, Path]:
    record = json.loads((DEPLOY_ROOT / "deployment.json").read_text(encoding="utf-8"))
    if record.get("commit") != commit:
        raise RuntimeError("deployment record belongs to another commit")
    return int(record["pid"]), Path(record["log"])


def process_is_running(pid: int) -> bool:
    try:
        os.kill(pid, 0)
    except OSError:
        return False
    stat = Path(f"/proc/{pid}/stat")
    return not stat.is_file() or stat.read_text(encoding="utf-8").split()[2] != "Z"


def wait_for_markers(commit: str, markers: tuple[str, ...], timeout: float) -> str:
    pid, log_path = deployment(commit)
    deadline = time.monotonic() + timeout
    content = ""
    while time.monotonic() < deadline:
        if not process_is_running(pid):
            raise RuntimeError(f"collector process {pid} exited during startup")
        if log_path.is_file():
            content = log_path.read_text(encoding="utf-8", errors="replace")
            fatal = next((marker for marker in FATAL_MARKERS if marker in content), None)
            if fatal:
                raise RuntimeError(f"collector reported fatal startup marker: {fatal}")
            if all(marker in content for marker in markers):
                return content
        time.sleep(0.25)
    missing = [marker for marker in markers if marker not in content]
    raise RuntimeError(f"collector startup did not produce markers: {missing}")


def main() -> int:
    if len(sys.argv) != 4:
        raise SystemExit("usage: deploy.py <start|readiness|smoke> <artifact-dir> <commit>")
    action, raw_artifact_dir, commit = sys.argv[1:]
    artifact_dir = Path(raw_artifact_dir).resolve()
    if action == "start":
        start(artifact_dir, commit)
    elif action == "readiness":
        wait_for_markers(commit, READINESS_MARKERS, timeout=10)
    elif action == "smoke":
        wait_for_markers(
            commit,
            READINESS_MARKERS + ("[cc.events.cybercity-test]", "collector-deploy-probe"),
            timeout=5,
        )
    else:
        raise SystemExit(f"unknown action: {action}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

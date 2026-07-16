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


def artifact_binary(artifact_dir: Path) -> Path:
    name = "cybercity-collector.exe" if os.name == "nt" else "cybercity-collector"
    candidate = artifact_dir / "target" / "release" / name
    if not candidate.is_file():
        raise RuntimeError(f"packaged binary is missing: {candidate}")
    return candidate


def start(artifact_dir: Path, commit: str) -> None:
    DEPLOY_ROOT.mkdir(exist_ok=True)
    source = artifact_binary(artifact_dir)
    target = DEPLOY_ROOT / source.name
    shutil.copy2(source, target)
    target.chmod(0o755)
    environment = os.environ.copy()
    environment.update(
        CCC_NODE_ID="collector-test",
        CCC_SERVICE_ID="cybercity-test",
        CCC_KAFKA_BROKER="127.0.0.1:9092",
        RUST_LOG="info",
    )
    log = (DEPLOY_ROOT / "collector.log").open("ab")
    process = subprocess.Popen(
        [str(target), str(ROOT / "config" / "example.toml")],
        cwd=ROOT,
        env=environment,
        stdin=subprocess.DEVNULL,
        stdout=log,
        stderr=subprocess.STDOUT,
        start_new_session=True,
    )
    (DEPLOY_ROOT / "deployment.json").write_text(
        json.dumps({"commit": commit, "pid": process.pid}) + "\n", encoding="utf-8"
    )


def assert_running(delay: float) -> None:
    time.sleep(delay)
    record = json.loads((DEPLOY_ROOT / "deployment.json").read_text(encoding="utf-8"))
    pid = int(record["pid"])
    try:
        os.kill(pid, 0)
    except OSError as error:
        raise RuntimeError(f"collector process {pid} is not running") from error
    if os.name != "nt" and Path(f"/proc/{pid}/stat").is_file():
        state = Path(f"/proc/{pid}/stat").read_text(encoding="utf-8").split()[2]
        if state == "Z":
            raise RuntimeError(f"collector process {pid} is a zombie")


def main() -> int:
    if len(sys.argv) != 4:
        raise SystemExit("usage: deploy.py <start|readiness|smoke> <artifact-dir> <commit>")
    action, raw_artifact_dir, commit = sys.argv[1:]
    artifact_dir = Path(raw_artifact_dir).resolve()
    if action == "start":
        start(artifact_dir, commit)
    elif action == "readiness":
        assert_running(1)
    elif action == "smoke":
        assert_running(3)
    else:
        raise SystemExit(f"unknown action: {action}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

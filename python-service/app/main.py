from __future__ import annotations

import os
from contextlib import asynccontextmanager
from datetime import datetime, timezone
from pathlib import Path

from fastapi import FastAPI


SERVICE_NAME = os.getenv("SERVICE_NAME", "python-service").strip() or "python-service"
SHARED_DIR = Path(os.getenv("SHARED_DIR", "/shared"))
PORT = int(os.getenv("PYTHON_SERVICE_PORT", "8000"))


def _marker_path() -> Path:
    return SHARED_DIR / f"{SERVICE_NAME}.txt"


def _append_record(marker: str) -> None:
    SHARED_DIR.mkdir(parents=True, exist_ok=True)
    timestamp = datetime.now(timezone.utc).isoformat()
    with _marker_path().open("a", encoding="utf-8") as handle:
        handle.write(f"{timestamp} | {SERVICE_NAME} | {marker}\n")


def _snapshot_shared() -> dict[str, str]:
    if not SHARED_DIR.exists():
        return {}

    snapshot: dict[str, str] = {}
    for path in sorted(SHARED_DIR.iterdir()):
        if path.is_file():
            snapshot[path.name] = path.read_text(encoding="utf-8").strip()
    return snapshot


@asynccontextmanager
async def lifespan(app: FastAPI):
    _append_record("boot")
    yield


app = FastAPI(title="Laboratory Work 11 Python Service", lifespan=lifespan)


@app.get("/health")
async def health() -> dict[str, str]:
    return {"service": SERVICE_NAME, "status": "ok"}


@app.post("/write")
async def write_shared() -> dict[str, str]:
    _append_record("manual write")
    return {"service": SERVICE_NAME, "status": "written"}


@app.get("/shared")
async def shared() -> dict[str, object]:
    return {"service": SERVICE_NAME, "files": _snapshot_shared()}


@app.get("/")
async def root() -> dict[str, object]:
    return {
        "service": SERVICE_NAME,
        "port": PORT,
        "shared_dir": str(SHARED_DIR),
    }

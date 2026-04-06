import asyncio
import sys
import shutil
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from app import main as app_module


@pytest.fixture()
def configured_app(monkeypatch):
    shared_dir = Path(__file__).resolve().parents[1] / ".test-shared"
    shared_dir.mkdir(parents=True, exist_ok=True)
    monkeypatch.setattr(app_module, "SERVICE_NAME", "python-service", raising=False)
    monkeypatch.setattr(app_module, "SHARED_DIR", shared_dir, raising=False)
    monkeypatch.setattr(app_module, "PORT", 8000, raising=False)

    yield app_module

    shutil.rmtree(shared_dir, ignore_errors=True)


def test_health(configured_app):
    assert asyncio.run(configured_app.health()) == {
        "service": "python-service",
        "status": "ok",
    }


def test_write_and_shared(configured_app):
    write_result = asyncio.run(configured_app.write_shared())
    assert write_result == {"service": "python-service", "status": "written"}

    shared = asyncio.run(configured_app.shared())
    assert "python-service.txt" in shared["files"]
    assert "manual write" in shared["files"]["python-service.txt"]


def test_lifespan_bootstraps_shared_file(configured_app):
    async def run_lifespan() -> None:
        async with configured_app.lifespan(configured_app.app):
            pass

    asyncio.run(run_lifespan())

    shared_file = Path(configured_app.SHARED_DIR) / "python-service.txt"
    assert shared_file.exists()
    assert "boot" in shared_file.read_text(encoding="utf-8")

"""Fixtures: serve dist/ (or use NK_BASE_URL), fail on console errors."""
from __future__ import annotations

import functools
import http.server
import os
import pathlib
import threading

import pytest

from nk import Game

ROOT = pathlib.Path(__file__).resolve().parents[2]


@pytest.fixture(scope="session")
def nk_url():
    """URL of the built game. Set NK_BASE_URL to test an existing server
    (e.g. http://localhost:8081/neon-knights/); otherwise dist/ (or NK_DIST)
    is served on a free local port for the session."""
    if os.environ.get("NK_BASE_URL"):
        yield os.environ["NK_BASE_URL"]
        return
    dist = pathlib.Path(os.environ.get("NK_DIST", ROOT / "dist"))
    if not (dist / "index.html").exists():
        pytest.exit(f"{dist}/index.html not found: run `trunk build --release` first", returncode=2)

    class Quiet(http.server.SimpleHTTPRequestHandler):
        def log_message(self, *args):
            pass

    server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), functools.partial(Quiet, directory=str(dist)))
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    yield f"http://127.0.0.1:{server.server_address[1]}/"
    server.shutdown()


@pytest.fixture
def browser_context_args(browser_context_args):
    return {**browser_context_args, "viewport": {"width": 1280, "height": 800}}


@pytest.fixture
def console(page):
    """Every console error/warning and uncaught page error; asserted empty
    when the test ends, so any of them fails the test."""
    problems: list[str] = []
    page.on("console", lambda m: problems.append(f"console.{m.type}: {m.text}") if m.type in ("error", "warning") else None)
    page.on("pageerror", lambda e: problems.append(f"pageerror: {e}"))
    yield problems
    assert not problems, "browser console problems:\n" + "\n".join(problems)


@pytest.fixture
def game(page, nk_url, console) -> Game:
    return Game(page, nk_url).open()


@pytest.fixture
def probed_game(page, nk_url, console) -> Game:
    return Game(page, nk_url).open(probe=True)

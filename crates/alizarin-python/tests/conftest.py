"""
No unexpected skips (PARITY-RESEARCH.md, D-4).

A test that skips because a fixture or handler went missing passes silently.
So a run fails if any test skips that isn't listed, with its reason, under
"skips" in tests/guard-allowlist.json (keyed by node id relative to this
crate, e.g. "tests/test_x.py::test_y[param]"). xfails don't count. Stale
entries aren't checked here, since a partial run (-k, one file) would make
every unlisted test look stale.
"""

import json
import pathlib

import pytest

_ALLOWLIST = pathlib.Path(__file__).resolve().parents[3] / "tests" / "guard-allowlist.json"
_CRATE = pathlib.Path(__file__).resolve().parents[1]
_skipped = []


def _node_id(report):
    path = pathlib.Path(str(report.fspath)).resolve()
    try:
        rel = path.relative_to(_CRATE).as_posix()
    except ValueError:
        return report.nodeid
    return rel + report.nodeid[report.nodeid.index("::"):] if "::" in report.nodeid else rel


def pytest_runtest_logreport(report):
    if report.skipped and not hasattr(report, "wasxfail"):
        _skipped.append(_node_id(report))


@pytest.hookimpl(trylast=True)
def pytest_sessionfinish(session, exitstatus):
    if not _ALLOWLIST.exists():
        return
    allowed = json.loads(_ALLOWLIST.read_text()).get("skips", {})
    unexpected = sorted(set(n for n in _skipped if n not in allowed))
    if unexpected:
        reporter = session.config.pluginmanager.get_plugin("terminalreporter")
        lines = ["Unexpected skips (list them in tests/guard-allowlist.json \"skips\" with a reason, or fix them):"]
        lines += [f"  {n}" for n in unexpected]
        if reporter:
            reporter.write_line("\n".join(lines), red=True)
        if session.exitstatus == 0:
            session.exitstatus = 1

#!/usr/bin/env python3
"""
Flag test patterns that let a test pass without checking anything (PARITY-RESEARCH.md, D).

Python tests (core and ext/*/python), by AST:
  swallowed-except  an except in a test that neither re-raises nor fails
  assert-under-if   asserts only inside an `if` whose else doesn't assert
  skip-in-body      pytest.skip() inside a test body
  no-assertion      a test with no assert, pytest.raises or pytest.fail
TS tests (tests/, ext/*/js/tests), by line:
  skip              describe/test/it .skip, .skipIf, .todo
  catch-swallows    a catch that neither rethrows nor asserts
  expect-under-if   expect() only inside an `if` with no else

Each finding has a key without line numbers (file::scope::kind::detail).
Today's findings are listed, with reasons, in tests/guard-allowlist.json
under "testIntegrity". The check fails on a finding not listed, and on a
listed key no longer found, so the list only shrinks: fix the test, then
delete its entry.

Usage: python3 scripts/guards/test_integrity.py [--current]
  --current  print today's findings as JSON, for (re)building the list
"""
import ast
import json
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
ALLOWLIST = ROOT / "tests" / "guard-allowlist.json"


def _short(node, limit=60):
    # Without quotes: ast.unparse quotes f-strings differently across Python
    # versions, and the keys must match on every interpreter CI uses.
    return re.sub(r"\s+", " ", re.sub(r"[\"']", "", ast.unparse(node)))[:limit]


def _calls_named(node, words):
    for n in ast.walk(node):
        if isinstance(n, ast.Call):
            name = ast.unparse(n.func).lower()
            if any(w in name for w in words):
                return True
    return False


def _checks(node):
    return any(isinstance(n, (ast.Assert, ast.Raise)) for n in ast.walk(node)) or _calls_named(
        node, ("assert", "fail", "raises", "expect", "verify")
    )


def _is_fixture(fn):
    return any("fixture" in ast.unparse(d) for d in fn.decorator_list)


def python_findings(path):
    rel = path.relative_to(ROOT).as_posix()
    tree = ast.parse(path.read_text(), str(path))
    for fn in ast.walk(tree):
        if not isinstance(fn, (ast.FunctionDef, ast.AsyncFunctionDef)):
            continue
        if not fn.name.startswith("test") or _is_fixture(fn):
            continue
        scope = f"{rel}::{fn.name}"
        for node in ast.walk(fn):
            if isinstance(node, ast.Try):
                for handler in node.handlers:
                    if not _checks(handler) and not _calls_named(handler, ("skip",)):
                        caught = _short(handler.type) if handler.type else "bare"
                        yield f"{scope}::swallowed-except::{caught}"
            elif isinstance(node, ast.If):
                if any(isinstance(n, ast.Assert) for s in node.body for n in ast.walk(s)) and not any(
                    _checks(s) for s in node.orelse
                ):
                    yield f"{scope}::assert-under-if::{_short(node.test)}"
            elif isinstance(node, ast.Call) and ast.unparse(node.func) in ("pytest.skip", "skip"):
                reason = _short(node.args[0]) if node.args else ""
                yield f"{scope}::skip-in-body::{reason}"
        if not _checks(fn):
            yield f"{scope}::no-assertion::"


TS_SKIP = re.compile(r"\b(?:describe|test|it)\.(?:skip|skipIf|todo)\b|\bskipIf\(")


def ts_findings(path):
    rel = path.relative_to(ROOT).as_posix()
    lines = path.read_text().splitlines()
    for i, line in enumerate(lines):
        s = line.strip()
        snippet = re.sub(r"\s+", " ", s)[:80]
        if TS_SKIP.search(s):
            yield f"{rel}::-::skip::{snippet}"
        if re.match(r"(\}\s*)?catch\b", s):
            body = " ".join(following.strip() for following in lines[i : i + 6])
            if not re.search(r"\b(throw|expect|assert|fail)\b", body):
                yield f"{rel}::-::catch-swallows::{snippet}"
        if re.match(r"if\s*\(", s):
            depth, j, block = 0, i, []
            while j < len(lines):
                block.append(lines[j])
                depth += lines[j].count("{") - lines[j].count("}")
                if depth <= 0 and j > i:
                    break
                j += 1
            has_else = any(re.match(r"\s*\}?\s*else\b", after) for after in lines[j : j + 2])
            if any("expect(" in b for b in block) and not has_else:
                yield f"{rel}::-::expect-under-if::{snippet}"


def current():
    found = set()
    py_dirs = [ROOT / "crates/alizarin-python/tests", *sorted(ROOT.glob("ext/*/python/tests"))]
    for d in py_dirs:
        for p in sorted(d.rglob("test_*.py")):
            found.update(python_findings(p))
    ts_dirs = [ROOT / "tests", *sorted(ROOT.glob("ext/*/js/tests"))]
    for d in ts_dirs:
        for p in sorted(d.rglob("*.test.ts")):
            if "node_modules" not in p.parts:
                found.update(ts_findings(p))
    return found


def main():
    found = current()
    if "--current" in sys.argv:
        print(json.dumps({k: "" for k in sorted(found)}, indent=2, ensure_ascii=False))
        return 0
    allowed = json.loads(ALLOWLIST.read_text()).get("testIntegrity", {})
    new = sorted(found - set(allowed))
    gone = sorted(set(allowed) - found)
    for key in new:
        print(f"NEW   {key}")
    for key in gone:
        print(f"FIXED {key}  (remove it from tests/guard-allowlist.json)")
    if new or gone:
        print(f"\n{len(new)} new finding(s), {len(gone)} stale allow-list entr(y/ies). See PARITY-RESEARCH.md, D.")
        return 1
    print(f"test integrity: {len(found)} known finding(s), none new")
    return 0


if __name__ == "__main__":
    sys.exit(main())

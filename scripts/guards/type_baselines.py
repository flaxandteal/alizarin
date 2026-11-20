#!/usr/bin/env python3
"""
Type-check baselines (PARITY-RESEARCH.md, C).

Runs mypy over the Python package and strict tsc over js/ and tests/, counts
errors by code, and compares with tests/guard-allowlist.json "typeBaselines".
A count above its baseline fails (new errors); a count below it also fails,
asking for the baseline to be lowered, so the numbers only go down.

Needs mypy importable by this interpreter (`pip install mypy`) and the repo's
node_modules (`npm ci`).

Usage: python3 scripts/guards/type_baselines.py [--mypy] [--tsc] [--current]
  with neither --mypy nor --tsc, runs both
  --current  print today's counts as JSON, for (re)building the baselines
"""
import collections
import json
import pathlib
import re
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
ALLOWLIST = ROOT / "tests" / "guard-allowlist.json"


def mypy_counts():
    out = subprocess.run(
        [
            sys.executable, "-m", "mypy",
            str(ROOT / "crates/alizarin-python/python/alizarin"),
            "--ignore-missing-imports", "--follow-imports=silent", "--no-error-summary",
        ],
        capture_output=True, text=True, cwd=ROOT,
    ).stdout
    return collections.Counter(re.findall(r"error: .*\[([a-z-]+)\]\s*$", out, re.M))


def tsc_counts():
    tsc = ROOT / "node_modules/.bin/tsc"
    out = subprocess.run(
        [str(tsc), "-p", str(ROOT / "tsconfig.json"), "--noEmit", "--strict"],
        capture_output=True, text=True, cwd=ROOT,
    ).stdout
    return collections.Counter(re.findall(r"error (TS\d+):", out))


def compare(name, counts, baseline):
    problems = []
    for code in sorted(set(counts) | set(baseline)):
        have, allowed = counts.get(code, 0), baseline.get(code, 0)
        if have > allowed:
            problems.append(f"{name} {code}: {have} errors, baseline {allowed} (new errors)")
        elif have < allowed:
            problems.append(f"{name} {code}: {have} errors, baseline {allowed} (lower the baseline)")
    return problems


def main():
    args = set(sys.argv[1:])
    run = {k for k in ("--mypy", "--tsc") if k in args} or {"--mypy", "--tsc"}
    counts = {}
    if "--mypy" in run:
        counts["mypy"] = mypy_counts()
    if "--tsc" in run:
        counts["tsc"] = tsc_counts()
    if "--current" in args:
        print(json.dumps({k: dict(sorted(v.items())) for k, v in counts.items()}, indent=2))
        return 0
    baselines = json.loads(ALLOWLIST.read_text()).get("typeBaselines", {})
    problems = []
    for name, c in counts.items():
        problems += compare(name, c, baselines.get(name, {}))
    for p in problems:
        print(p)
    if problems:
        print("\nSee PARITY-RESEARCH.md, C. Baselines: tests/guard-allowlist.json typeBaselines.")
        return 1
    print("type baselines: " + ", ".join(f"{k} {sum(v.values())} error(s), as baselined" for k, v in counts.items()))
    return 0


if __name__ == "__main__":
    sys.exit(main())

#!/usr/bin/env python3
"""
No extension binding links alizarin-core (notes/extension-interface-split.md).

An extension module (.so, .node, .wasm) is loaded beside the host, which has
its own alizarin-core; a copy of core in the extension has its own, empty,
globals (current language, graph, extension and RDM registries), so whatever
it reads from them is silently wrong. Extensions build against
alizarin-extension-api and reach the host's state through its callbacks.

For every ext/*/{python,napi,wasm} crate, `cargo tree -e normal` (all
targets) must not list alizarin-core. There are no exceptions to allow.

Usage: python3 scripts/guards/no_second_core.py   (needs cargo)
"""
import pathlib
import re
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
BINDINGS = ("python", "napi", "wasm")


def links_core(manifest: pathlib.Path) -> bool:
    tree = subprocess.run(
        ["cargo", "tree", "-q", "-e", "normal", "--target", "all",
         "--manifest-path", str(manifest)],
        capture_output=True, text=True, check=True,
    ).stdout
    # A dependency line reads "├── alizarin-core v2.0.0-alpha.148 (...)".
    return re.search(r"(^|\s)alizarin-core v", tree, re.MULTILINE) is not None


def main() -> int:
    crates = sorted(
        manifest
        for binding in BINDINGS
        for manifest in ROOT.glob(f"ext/*/{binding}/Cargo.toml")
    )
    offenders = [m.parent.relative_to(ROOT) for m in crates if links_core(m)]
    for crate in offenders:
        print(f"LINKS CORE  {crate}")
    print(f"no second core: {len(crates)} extension binding(s), {len(offenders)} linking alizarin-core")
    return 1 if offenders else 0


if __name__ == "__main__":
    sys.exit(main())

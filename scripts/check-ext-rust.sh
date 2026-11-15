#!/usr/bin/env bash
# Check every extension crate (ext/*/{core,napi,python,wasm}) with rustfmt and
# clippy, as the workspace's own pre-commit and CI checks do for crates/. The
# extension crates sit outside the workspace, with their own lockfiles, so
# `cargo fmt --all` and `cargo clippy --workspace` never see them; without this
# they drift.
#
# Runs every check, lists each crate that fails, and exits non-zero if any do.
# Used by .pre-commit-config.yaml (when ext/ Rust files change) and CI.
#
#   bash scripts/check-ext-rust.sh            # fmt + clippy
#   bash scripts/check-ext-rust.sh --fmt-only # rustfmt only
set -uo pipefail

cd "$(dirname "$0")/.."

fmt_only=false
if [ "${1:-}" = "--fmt-only" ]; then
  fmt_only=true
fi

failed=()
for manifest in ext/*/core/Cargo.toml ext/*/napi/Cargo.toml ext/*/python/Cargo.toml ext/*/wasm/Cargo.toml; do
  [ -f "$manifest" ] || continue
  crate=$(dirname "$manifest")
  if ! cargo fmt --manifest-path "$manifest" -- --check >/dev/null 2>&1; then
    failed+=("$crate: rustfmt (run: cargo fmt --manifest-path $manifest)")
  fi
  if [ "$fmt_only" = false ]; then
    if ! cargo clippy --quiet --manifest-path "$manifest" -- -D warnings >/dev/null 2>&1; then
      failed+=("$crate: clippy (run: cargo clippy --manifest-path $manifest -- -D warnings)")
    fi
  fi
done

if [ ${#failed[@]} -gt 0 ]; then
  echo "Extension crates need attention:"
  printf '  %s\n' "${failed[@]}"
  exit 1
fi
echo "Extension crates: formatted and lint-free."

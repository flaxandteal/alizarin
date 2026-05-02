#!/usr/bin/env bash
# Run all tests across the alizarin workspace: Rust, WASM+JS, NAPI, Python, Extensions.
# Mirrors the CI workflows in .github/workflows/.
#
# Usage:
#   ./utils/run-all-tests.sh          # Run everything
#   ./utils/run-all-tests.sh rust     # Just Rust (clippy + tests)
#   ./utils/run-all-tests.sh wasm     # WASM build + JS tests
#   ./utils/run-all-tests.sh napi     # NAPI build + tests
#   ./utils/run-all-tests.sh python   # Python (maturin) build + pytest
#   ./utils/run-all-tests.sh ext-js   # JS extensions build
#   ./utils/run-all-tests.sh ext-py   # Python extensions build + test

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BOLD='\033[1m'
NC='\033[0m'

FAILED=()
PASSED=()

run_section() {
    local name="$1"
    shift
    echo ""
    echo -e "${BOLD}════════════════════════════════════════════════════════════════${NC}"
    echo -e "${BOLD}  $name${NC}"
    echo -e "${BOLD}════════════════════════════════════════════════════════════════${NC}"
    echo ""
    if "$@"; then
        PASSED+=("$name")
        echo -e "${GREEN}✓ $name passed${NC}"
    else
        FAILED+=("$name")
        echo -e "${RED}✗ $name FAILED${NC}"
    fi
}

# ─── Rust ────────────────────────────────────────────────────────────────────
test_rust() {
    run_section "Rust fmt" cargo fmt --all -- --check
    run_section "Rust clippy" cargo clippy --workspace --exclude alizarin-python -- -D warnings
    run_section "Rust tests" cargo test --workspace --exclude alizarin-python
}

# ─── WASM + JS ───────────────────────────────────────────────────────────────
test_wasm() {
    run_section "WASM build" bash -c \
        'cd crates/alizarin-wasm && wasm-pack build --target web --out-dir ../../pkg --out-name alizarin --dev'
    run_section "JS install" npm ci
    run_section "JS build" npm run build:js
    run_section "JS tests" npm test
    run_section "JS lint" npx eslint . --ext .ts,.tsx
}

# ─── NAPI ────────────────────────────────────────────────────────────────────
test_napi() {
    run_section "NAPI install" bash -c 'cd crates/alizarin-napi && npm install'
    run_section "NAPI build" bash -c 'cd crates/alizarin-napi && npm run build'
    run_section "NAPI tests" bash -c 'cd crates/alizarin-napi && npm test'
    # vitest with napi config is run from repo root
    if [ -f vitest.napi.config.js ]; then
        run_section "NAPI vitest" npx vitest run --config vitest.napi.config.js
    elif [ -f vitest.napi.config.ts ]; then
        run_section "NAPI vitest" npx vitest run --config vitest.napi.config.ts
    fi
}

# ─── Python ──────────────────────────────────────────────────────────────────
test_python() {
    # Use existing .venv if present; otherwise create a temp one.
    if [ -d "$REPO_ROOT/.venv" ]; then
        local venv="$REPO_ROOT/.venv"
    else
        local venv="/tmp/alizarin-test-venv"
        run_section "Python venv" bash -c \
            "python3 -m venv '$venv' && source '$venv/bin/activate' && pip install pytest pytest-asyncio maturin"
    fi
    local activate="source '$venv/bin/activate'"
    run_section "Python build" bash -c \
        "$activate && cd crates/alizarin-python && maturin build --release --out dist"
    run_section "Python install" bash -c \
        "$activate && pip install --force-reinstall crates/alizarin-python/dist/*.whl"
    run_section "Python tests" bash -c \
        "$activate && cd crates/alizarin-python && pytest tests/ -v"
}

# ─── JS Extensions ──────────────────────────────────────────────────────────
test_ext_js() {
    for ext_dir in ext/js/@alizarin/*/; do
        [ -d "$ext_dir" ] || continue
        ext_name=$(basename "$ext_dir")
        run_section "JS ext: $ext_name build" bash -c \
            "cd '$ext_dir' && npm install && npm run build"
        if [ -d "$ext_dir/tests" ]; then
            run_section "JS ext: $ext_name tests" bash -c \
                "npm test -- '$ext_dir/tests/'"
        fi
    done
}

# ─── Python Extensions ──────────────────────────────────────────────────────
test_ext_py() {
    # Reuse the same venv as test_python
    if [ -d "$REPO_ROOT/.venv" ]; then
        local venv="$REPO_ROOT/.venv"
    else
        local venv="/tmp/alizarin-test-venv"
    fi
    local activate="source '$venv/bin/activate'"

    for ext_dir in ext/python/*/; do
        [ -d "$ext_dir" ] || continue
        ext_name=$(basename "$ext_dir")
        run_section "Py ext: $ext_name build" bash -c \
            "$activate && cd '$ext_dir' && maturin build --release --out dist"
        run_section "Py ext: $ext_name install" bash -c \
            "$activate && pip install --no-deps --force-reinstall '$ext_dir'/dist/*.whl"
        if [ -d "$ext_dir/tests" ]; then
            run_section "Py ext: $ext_name tests" bash -c \
                "$activate && cd /tmp && pytest '$REPO_ROOT/$ext_dir/tests' -v --import-mode=importlib"
        fi
    done
}

# ─── Main ────────────────────────────────────────────────────────────────────
targets="${1:-all}"

case "$targets" in
    all)
        test_rust
        test_wasm
        test_napi
        test_python
        test_ext_js
        test_ext_py
        ;;
    rust)    test_rust ;;
    wasm)    test_wasm ;;
    napi)    test_napi ;;
    python)  test_python ;;
    ext-js)  test_ext_js ;;
    ext-py)  test_ext_py ;;
    *)
        echo "Usage: $0 [all|rust|wasm|napi|python|ext-js|ext-py]"
        exit 1
        ;;
esac

# ─── Summary ─────────────────────────────────────────────────────────────────
echo ""
echo -e "${BOLD}════════════════════════════════════════════════════════════════${NC}"
echo -e "${BOLD}  SUMMARY${NC}"
echo -e "${BOLD}════════════════════════════════════════════════════════════════${NC}"
echo ""
for s in "${PASSED[@]+"${PASSED[@]}"}"; do
    echo -e "  ${GREEN}✓${NC} $s"
done
for s in "${FAILED[@]+"${FAILED[@]}"}"; do
    echo -e "  ${RED}✗${NC} $s"
done
echo ""

if [ ${#FAILED[@]} -gt 0 ]; then
    echo -e "${RED}${#FAILED[@]} section(s) failed.${NC}"
    exit 1
else
    echo -e "${GREEN}All sections passed.${NC}"
fi

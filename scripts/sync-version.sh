#!/bin/bash
# Sync version from VERSION file to all package manifests
# Usage: ./scripts/sync-version.sh [version]
# If version is provided, update VERSION file first

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"

# If version provided as argument, update VERSION file
if [ -n "$1" ]; then
    echo "$1" > "$ROOT_DIR/VERSION"
fi

VERSION=$(cat "$ROOT_DIR/VERSION" | tr -d '\n')

if [ -z "$VERSION" ]; then
    echo "Error: VERSION file is empty or missing"
    exit 1
fi

echo "Syncing version $VERSION across all packages..."

# Convert semver to Cargo-compatible (no hyphens in prerelease for some tools)
# Rust accepts: 0.2.1-alpha.5 or 0.2.1-alpha5
CARGO_VERSION="$VERSION"

# Update package.json
if [ -f "$ROOT_DIR/package.json" ]; then
    # Use node for reliable JSON editing
    node -e "
        const fs = require('fs');
        const pkg = JSON.parse(fs.readFileSync('$ROOT_DIR/package.json', 'utf8'));
        pkg.version = '$VERSION';
        fs.writeFileSync('$ROOT_DIR/package.json', JSON.stringify(pkg, null, 2) + '\n');
    "
    echo "  ✓ package.json"
fi

# Update root Cargo.toml (workspace package version)
if [ -f "$ROOT_DIR/Cargo.toml" ]; then
    sed -i "0,/^version = /s/^version = .*/version = \"$CARGO_VERSION\"/" "$ROOT_DIR/Cargo.toml"
    echo "  ✓ Cargo.toml (root)"
fi

# Update crate Cargo.toml files
for cargo_file in "$ROOT_DIR"/crates/*/Cargo.toml; do
    if [ -f "$cargo_file" ]; then
        crate_name=$(basename "$(dirname "$cargo_file")")
        sed -i "0,/^version = /s/^version = .*/version = \"$CARGO_VERSION\"/" "$cargo_file"
        echo "  ✓ crates/$crate_name/Cargo.toml"
    fi
done

# Update Python pyproject.toml if it exists
# Convert to PEP 440 format: 0.2.1-alpha.12 -> 0.2.1a12
PEP440_VERSION=$(echo "$VERSION" | sed 's/-alpha\./a/' | sed 's/-beta\./b/' | sed 's/-rc\./rc/')
if [ -f "$ROOT_DIR/crates/alizarin-python/pyproject.toml" ]; then
    sed -i "s/^version = .*/version = \"$PEP440_VERSION\"/" "$ROOT_DIR/crates/alizarin-python/pyproject.toml"
    echo "  ✓ crates/alizarin-python/pyproject.toml"
fi

# Update Python __version__ if it exists
for py_init in "$ROOT_DIR"/python/*/alizarin/__init__.py "$ROOT_DIR"/crates/alizarin-python/python/alizarin/__init__.py; do
    if [ -f "$py_init" ]; then
        sed -i "s/__version__ = .*/__version__ = \"$VERSION\"/" "$py_init"
        echo "  ✓ $(basename $(dirname $(dirname "$py_init")))/alizarin/__init__.py"
    fi
done

# Update all JS extensions in ext/js/@alizarin/*/package.json
for ext_pkg in "$ROOT_DIR"/ext/js/@alizarin/*/package.json; do
    if [ -f "$ext_pkg" ]; then
        ext_name=$(basename "$(dirname "$ext_pkg")")
        node -e "
            const fs = require('fs');
            const pkg = JSON.parse(fs.readFileSync('$ext_pkg', 'utf8'));
            pkg.version = '$VERSION';
            pkg.peerDependencies = pkg.peerDependencies || {};
            pkg.peerDependencies.alizarin = '$VERSION';
            fs.writeFileSync('$ext_pkg', JSON.stringify(pkg, null, 2) + '\n');
        "
        echo "  ✓ ext/js/@alizarin/$ext_name/package.json (version + peerDependencies)"
    fi
done

# Update all Python extensions in ext/python/*/pyproject.toml
for ext_pyproject in "$ROOT_DIR"/ext/python/*/pyproject.toml; do
    if [ -f "$ext_pyproject" ]; then
        ext_name=$(basename "$(dirname "$ext_pyproject")")
        # Update version (use PEP 440 format)
        sed -i "s/^version = .*/version = \"$PEP440_VERSION\"/" "$ext_pyproject"
        # Update alizarin dependency version if present (use PEP 440 format)
        sed -i "s/\"alizarin>=.*\"/\"alizarin>=$PEP440_VERSION\"/" "$ext_pyproject"
        echo "  ✓ ext/python/$ext_name/pyproject.toml (version + dependencies)"
    fi
done

echo ""
echo "Version synced to $VERSION"

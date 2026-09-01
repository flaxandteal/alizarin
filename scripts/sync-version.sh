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

# (ext crates are handled generically by the ext/<name>/{core,python,js} loop below.)

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

# Update NAPI package.json
if [ -f "$ROOT_DIR/crates/alizarin-napi/package.json" ]; then
    node -e "
        const fs = require('fs');
        const pkg = JSON.parse(fs.readFileSync('$ROOT_DIR/crates/alizarin-napi/package.json', 'utf8'));
        pkg.version = '$VERSION';
        fs.writeFileSync('$ROOT_DIR/crates/alizarin-napi/package.json', JSON.stringify(pkg, null, 2) + '\n');
    "
    echo "  ✓ crates/alizarin-napi/package.json"
fi

# Update filelist JS extension
if [ -f "$ROOT_DIR/ext/filelist/js/package.json" ]; then
    node -e "
        const fs = require('fs');
        const pkg = JSON.parse(fs.readFileSync('$ROOT_DIR/ext/filelist/js/package.json', 'utf8'));
        pkg.version = '$VERSION';
        pkg.peerDependencies = pkg.peerDependencies || {};
        pkg.peerDependencies.alizarin = '$VERSION';
        fs.writeFileSync('$ROOT_DIR/ext/filelist/js/package.json', JSON.stringify(pkg, null, 2) + '\n');
    "
    echo "  ✓ ext/filelist/js/package.json (version + peerDependencies)"
fi

# Update filelist Cargo.toml files (core, wasm, python)
for filelist_cargo in "$ROOT_DIR"/ext/filelist/*/Cargo.toml; do
    if [ -f "$filelist_cargo" ]; then
        component_name=$(basename "$(dirname "$filelist_cargo")")
        sed -i "0,/^version = /s/^version = .*/version = \"$CARGO_VERSION\"/" "$filelist_cargo"
        echo "  ✓ ext/filelist/$component_name/Cargo.toml"
    fi
done

# Update filelist Python extension pyproject.toml
if [ -f "$ROOT_DIR/ext/filelist/python/pyproject.toml" ]; then
    sed -i "s/^version = .*/version = \"$PEP440_VERSION\"/" "$ROOT_DIR/ext/filelist/python/pyproject.toml"
    sed -i "s/\"alizarin>=.*\"/\"alizarin>=$PEP440_VERSION\"/" "$ROOT_DIR/ext/filelist/python/pyproject.toml"
    echo "  ✓ ext/filelist/python/pyproject.toml (version + dependencies)"
fi

# Update extensions in ext/<name>/{core,js,python}/ layout (e.g. alizarin-clm, alizarin-pg)
for ext_dir in "$ROOT_DIR"/ext/*/; do
    ext_name=$(basename "$ext_dir")
    # Skip already-handled extensions
    [ "$ext_name" = "filelist" ] && continue
    [ "$ext_name" = "js" ] && continue
    [ "$ext_name" = "python" ] && continue

    # JS package.json
    if [ -f "$ext_dir/js/package.json" ]; then
        node -e "
            const fs = require('fs');
            const pkg = JSON.parse(fs.readFileSync('$ext_dir/js/package.json', 'utf8'));
            pkg.version = '$VERSION';
            pkg.peerDependencies = pkg.peerDependencies || {};
            pkg.peerDependencies.alizarin = '$VERSION';
            fs.writeFileSync('$ext_dir/js/package.json', JSON.stringify(pkg, null, 2) + '\n');
        "
        echo "  ✓ ext/$ext_name/js/package.json (version + peerDependencies)"
    fi

    # Core Cargo.toml
    if [ -f "$ext_dir/core/Cargo.toml" ]; then
        sed -i "0,/^version = /s/^version = .*/version = \"$CARGO_VERSION\"/" "$ext_dir/core/Cargo.toml"
        echo "  ✓ ext/$ext_name/core/Cargo.toml"
    fi

    # Python Cargo.toml
    if [ -f "$ext_dir/python/Cargo.toml" ]; then
        sed -i "0,/^version = /s/^version = .*/version = \"$CARGO_VERSION\"/" "$ext_dir/python/Cargo.toml"
        echo "  ✓ ext/$ext_name/python/Cargo.toml"
    fi

    # Python pyproject.toml
    if [ -f "$ext_dir/python/pyproject.toml" ]; then
        sed -i "s/^version = .*/version = \"$PEP440_VERSION\"/" "$ext_dir/python/pyproject.toml"
        sed -i "s/\"alizarin>=.*\"/\"alizarin>=$PEP440_VERSION\"/" "$ext_dir/python/pyproject.toml"
        echo "  ✓ ext/$ext_name/python/pyproject.toml (version + dependencies)"
    fi
done

# Keep every INTRA-WORKSPACE dependency in lockstep with the workspace version.
# Publishable crates cannot carry a path-only dep (crates.io rejects it), so they
# declare a dual `{ path = "...", version = "..." }`; the `version` must track the
# workspace or the published crate pins a stale sibling and `cargo publish` fails
# to resolve it. `path =` is the discriminator - external crates.io deps never
# carry it - so any single-line inline-table dep with both `path` and `version`
# is one of ours. This covers alizarin-core's dep on alizarin-extension-api,
# every ext-core's dep on alizarin-core, and any future sibling dep, with no
# per-crate list to maintain.
for cargo_file in \
    "$ROOT_DIR"/Cargo.toml \
    "$ROOT_DIR"/crates/*/Cargo.toml \
    "$ROOT_DIR"/ext/*/*/Cargo.toml; do
    [ -f "$cargo_file" ] || continue
    # A non-comment line carrying BOTH path and version (either order).
    if grep -qE '^[^#]*path *=[^#]*version *=|^[^#]*version *=[^#]*path *=' "$cargo_file"; then
        # Bump the version on every path-bearing (non-comment) dep line.
        sed -i -E '/^[[:space:]]*#/!{ /path *=/ s/(version *= *)"[^"]*"/\1"'"$CARGO_VERSION"'"/ }' "$cargo_file"
        echo "  ✓ ${cargo_file#$ROOT_DIR/} (intra-workspace path+version deps)"
    fi
done

# Set the [package] version on EVERY ext subcrate. The per-extension loops above
# only cover core/ + python/ (and filelist's own glob); this catches napi/ and
# wasm/, which are otherwise never bumped and drift behind the workspace.
for cargo_file in "$ROOT_DIR"/ext/*/*/Cargo.toml; do
    [ -f "$cargo_file" ] || continue
    sed -i "0,/^version = /s/^version = .*/version = \"$CARGO_VERSION\"/" "$cargo_file"
    echo "  ✓ ${cargo_file#$ROOT_DIR/} (package version)"
done

# Normalise the npm side: the base @alizarin/napi package and every ext js/ +
# napi/ package. Set each package's own version, and re-pin any intra-project
# dependency (alizarin, @alizarin/*) that is on the beta line. The `-beta.`
# discriminator deliberately leaves the per-platform binary optionalDependencies
# (`@alizarin/napi-<triple>`, a separate `-alpha.` version line) and non-pins
# (`file:`, `^x`, `*`, `workspace:`) untouched.
for pkg_json in \
    "$ROOT_DIR"/crates/alizarin-napi/package.json \
    "$ROOT_DIR"/ext/*/js/package.json \
    "$ROOT_DIR"/ext/*/napi/package.json; do
    [ -f "$pkg_json" ] || continue
    VERSION="$VERSION" node -e "
        const fs = require('fs'), f = process.argv[1];
        const pkg = JSON.parse(fs.readFileSync(f, 'utf8'));
        pkg.version = process.env.VERSION;
        for (const field of ['dependencies','peerDependencies','optionalDependencies','devDependencies']) {
            const deps = pkg[field];
            if (!deps) continue;
            for (const [name, spec] of Object.entries(deps)) {
                if ((name === 'alizarin' || name.startsWith('@alizarin/')) &&
                    typeof spec === 'string' && spec.includes('-beta.')) {
                    deps[name] = process.env.VERSION;
                }
            }
        }
        fs.writeFileSync(f, JSON.stringify(pkg, null, 2) + '\n');
    " "$pkg_json"
    echo "  ✓ ${pkg_json#$ROOT_DIR/} (version + @alizarin beta pins)"
done

echo ""
echo "Version synced to $VERSION"

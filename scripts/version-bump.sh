#!/usr/bin/env bash
# version-bump.sh — Bump the shabda version.
#
# VERSION is the source of truth: cyrius.cyml reads `version = "${file:VERSION}"`,
# so bumping means writing VERSION and regenerating the distlib bundle
# (dist/shabda.cyr) so its `# Version:` header carries the new value.
#
# ⛔ REWRITTEN 2026-09-01 (v3.0.4). This script was still the Rust-era one and had been
# BROKEN since the port: after writing VERSION it ran
#     sed -i "s/^version = .*/.../" "$REPO_ROOT/Cargo.toml"
#     cargo generate-lockfile
# and `Cargo.toml` now lives under `rust-old/`, not the repo root. With `set -euo pipefail`
# the sed failed and the script DIED THERE — so `VERSION` was written and the bundle was
# never regenerated, leaving dist/shabda.cyr stamped with the previous version while
# VERSION said otherwise. It failed loudly, which is the only reason it was caught; a
# `|| true` on that line would have made it silent.
set -euo pipefail

[ $# -ne 1 ] && echo "Usage: $0 <version>" && exit 1
NEW_VERSION="$1"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "$NEW_VERSION" > "$REPO_ROOT/VERSION"

# Regenerate dist/shabda.cyr (+ .deps) so the bundle carries the new version.
# --all covers every [lib.*] profile, not just the base bundle: a source fix reaching the
# main bundle and none of the profiles is exactly the sankoch 2.7.6 incident.
cd "$REPO_ROOT" && cyrius distlib --all

echo
echo "Bumped to ${NEW_VERSION}. Next:"
echo "  - Update CHANGELOG.md (Keep a Changelog: Added/Changed/Fixed/Removed)"
echo "  - Commit, then tag: git tag v${NEW_VERSION}"

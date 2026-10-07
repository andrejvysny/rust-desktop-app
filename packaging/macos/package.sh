#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."
staging=$(mktemp -d)
trap 'rm -rf "$staging"' EXIT
bundle="$staging/Application Template.app"
mkdir -p "$bundle/Contents/MacOS" "$bundle/Contents/Resources" dist
cp "${CARGO_TARGET_DIR:-target}/release/app" "$bundle/Contents/MacOS/"
cp packaging/macos/Info.plist "$bundle/Contents/"
cp LICENSE-MIT LICENSE-APACHE NOTICE ATTRIBUTION.md "$bundle/Contents/Resources/"
cp "${CARGO_TARGET_DIR:-target}/release/app-cli" "$staging/"
ditto -c -k --keepParent "$bundle" "dist/app-template-macos-$(uname -m).zip"
cp "$staging/app-cli" "dist/app-cli-macos-$(uname -m)"

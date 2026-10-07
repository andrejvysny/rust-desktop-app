#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."
# Keep staging isolated so packaging never removes an existing dist directory.
staging=$(mktemp -d)
trap 'rm -rf "$staging"' EXIT
mkdir -p "$staging/app-template/bin" "$staging/app-template/share/applications" dist
cp "${CARGO_TARGET_DIR:-target}/release/app" "${CARGO_TARGET_DIR:-target}/release/app-cli" "$staging/app-template/bin/"
cp packaging/linux/app-template.desktop "$staging/app-template/share/applications/"
cp LICENSE-MIT LICENSE-APACHE NOTICE ATTRIBUTION.md "$staging/app-template/"
tar -czf "dist/app-template-linux-$(uname -m).tar.gz" -C "$staging" app-template

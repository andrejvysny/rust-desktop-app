#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../apps/app-web"
trunk build --release --dist ../../dist/web

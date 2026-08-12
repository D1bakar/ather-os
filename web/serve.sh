#!/usr/bin/env bash
# One-command Universal Platform launcher (run from repository root: ./web/serve.sh)
set -euo pipefail

WEB_ROOT="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$WEB_ROOT/.." && pwd)"

cd "$REPO_ROOT"

if [[ ! -f "$WEB_ROOT/public/manifest.json" ]]; then
  echo "==> manifest.json missing; building web artifacts"
  if command -v pwsh >/dev/null 2>&1; then
    pwsh -ExecutionPolicy Bypass -File "$REPO_ROOT/scripts/build-web-artifacts.ps1"
  else
    powershell -ExecutionPolicy Bypass -File "$REPO_ROOT/scripts/build-web-artifacts.ps1"
  fi
fi

cd "$WEB_ROOT"

if [[ ! -d node_modules ]]; then
  echo "==> npm install"
  npm install
fi

echo "==> Serving http://localhost:8080 (Ctrl+C to stop)"
npm run serve

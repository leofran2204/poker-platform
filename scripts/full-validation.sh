#!/usr/bin/env bash
# Manual local gameplay validation. Shared implementation owns isolation and reporting.
set -euo pipefail
if [[ "${FULL_VALIDATION_APPROVED:-}" != 1 ]]; then
  echo "Explicit authorization required: FULL_VALIDATION_APPROVED=1" >&2
  exit 2
fi
if ! command -v cargo >/dev/null 2>&1 && [[ -f "$HOME/.cargo/env" ]]; then
  source "$HOME/.cargo/env"
fi
project_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
exec python3 "$project_root/scripts/full-validation.py" "${1:-all}" --minutes "${FULL_VALIDATION_MINUTES:-60}"

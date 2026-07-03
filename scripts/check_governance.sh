#!/usr/bin/env bash
set -euo pipefail

required_files=(
  ".github/CODEOWNERS"
  ".github/pull_request_template.md"
  ".github/workflows/ci.yml"
  ".github/workflows/governance.yml"
  ".github/workflows/release-evidence.yml"
  "docs/model_validation.md"
  "docs/security_review.md"
  "docs/operations.md"
  "docs/observability.md"
  "docs/schemas.md"
  "docs/benchmark_matrix.md"
  "scripts/release_evidence.sh"
  "scripts/check_governance.sh"
)

for path in "${required_files[@]}"; do
  if [[ ! -f "${path}" ]]; then
    echo "missing required governance file: ${path}" >&2
    exit 1
  fi
done

grep -q "@gregorian-09" .github/CODEOWNERS
grep -q "External reviewer approval is required before merge" .github/pull_request_template.md
grep -q "Independent validator" docs/model_validation.md
grep -q "named reviewer completes the required actions" docs/security_review.md
grep -q "target trading server" docs/benchmark_matrix.md

if [[ "${GITHUB_EVENT_NAME:-}" == "pull_request" ]]; then
  body="${PR_BODY:-}"
  for marker in \
    "External reviewer approval is required before merge" \
    "cargo test --workspace --all-features" \
    "cargo clippy --workspace --all-targets --all-features -- -D warnings"; do
    if [[ "${body}" != *"${marker}"* ]]; then
      echo "pull request body is missing governance marker: ${marker}" >&2
      exit 1
    fi
  done
fi

echo "governance checks passed"

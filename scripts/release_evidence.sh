#!/usr/bin/env bash
set -euo pipefail

out_dir="${1:-target/release-evidence}"
mkdir -p "${out_dir}"

run_and_capture() {
  local name="$1"
  shift
  {
    echo "$ $*"
    "$@"
  } >"${out_dir}/${name}.log" 2>&1
}

{
  echo "commit=$(git rev-parse HEAD)"
  echo "branch=$(git rev-parse --abbrev-ref HEAD)"
  echo "rustc=$(rustc --version)"
  echo "cargo=$(cargo --version)"
  echo "generated_at_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
} >"${out_dir}/metadata.txt"

run_and_capture fmt cargo fmt --all --check
run_and_capture test cargo test --workspace --all-features
run_and_capture examples cargo test --workspace --examples --all-features
run_and_capture clippy cargo clippy --workspace --all-targets --all-features -- -D warnings
run_and_capture docs env RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps
run_and_capture audit cargo audit --db target/advisory-db
run_and_capture deny cargo deny check
run_and_capture package-risk-core cargo package -p risk-core --allow-dirty
run_and_capture benchmark-smoke cargo run -p risk-bench --release -- --iterations 5000
run_and_capture governance scripts/check_governance.sh

{
  echo "# Release Evidence"
  echo
  cat "${out_dir}/metadata.txt"
  echo
  echo "## Logs"
  for log in "${out_dir}"/*.log; do
    echo "- $(basename "${log}")"
  done
} >"${out_dir}/README.md"

echo "release evidence written to ${out_dir}"

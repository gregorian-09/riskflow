# Release Governance

This repository uses GitHub review controls to make institutional sign-off
auditable without pretending CI can replace human review.

## Pull Requests

Required repository settings:

1. Enable branch protection for `master` or `main`.
2. Require status checks:
   - `rust`,
   - `governance`.
3. Require pull request review before merging.
4. Require review from Code Owners.
5. Dismiss stale approvals when new commits are pushed.
6. Require conversation resolution before merging.

The `CODEOWNERS` file assigns sensitive paths to `@gregorian-09`, making
that review mandatory when branch protection has Code Owner review enabled.

## Release Evidence

The `release evidence` workflow creates a release artifact containing:

- commit and toolchain metadata,
- formatting output,
- full workspace tests,
- example tests,
- Clippy output,
- rustdoc output,
- audit and deny output,
- package verification,
- benchmark smoke output,
- governance check output.

## Human Approval Gates

The release workflow uses protected GitHub Environments:

- `model-validation`,
- `security-review`.

Configure these environments in GitHub with required reviewers. A release
cannot pass those jobs until the required reviewer approves the environment
deployment.

## Production Benchmark Gate

The `benchmark matrix` workflow targets a self-hosted runner labeled
`risk-prod-bench`. Attach that label only to approved production-like benchmark
hardware. The workflow uploads the environment and benchmark result files as
evidence for `docs/benchmark_matrix.md`.

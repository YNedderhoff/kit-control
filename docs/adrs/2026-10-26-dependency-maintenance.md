# Monthly dependency maintenance with hosted Renovate

- Status: accepted.
- Decision date: 2026-10-26.
- Decision basis: owner selection of hosted Renovate, monthly updates, local notice generation with CI verification, and required PR checks.

## Context

KitControl ships a Cargo workspace and an npm frontend, builds through SHA-pinned GitHub Actions, and pins Rust and Node. Updates can invalidate both license notices and distributed MPL source archives. Compiler success alone does not detect that failure.

## Decision

Use the hosted Renovate GitHub App, restricted to this repository. It creates update PRs; maintainers review and merge them manually. Use the first seven days of each month in Europe/Berlin for new regular branches and a seven-day release age where timestamps exist. Open security-fix PRs independently of those limits. Retain GitHub's dependency graph and Dependabot alerts, with Dependabot update PRs disabled.

Group routine Cargo/frontend updates separately, couple the Tauri Rust/npm packages, and separate other major upgrades, potentially breaking pre-1.0 upgrades and toolchain changes. Refresh transitive dependencies through monthly lockfile maintenance. Release-age checks do not guarantee the age of every transitive dependency.

Generate notices and required Cargo MPL sources locally with one command, then verify them in read-only CI. Keep the existing license generators and use established TOML, template and SPDX libraries. Custom code only orchestrates them and reconciles the source archives against the lockfile. Do not grant CI write tokens or enable follow-up commits. Block npm licenses requiring MPL source distribution pending a separately reviewed distribution path; permitting MPL for Rust does not implement npm source delivery.

Require PRs, an up-to-date branch and successful dependency/platform checks on `main`, without a mandatory additional reviewer or a Renovate bypass. Keep auto-merge disabled.

## Alternatives and consequences

Dependabot supports the main package ecosystems, Rust toolchains and cross-ecosystem grouping, with less setup. Renovate also covers `.nvmrc` and provides one policy across the chosen managers. Self-hosting would add token, runner and bot maintenance without a current requirement. Hosted arbitrary post-upgrade scripts are restricted, so local generation plus CI verification is the selected approach.

Maintainers still assess migrations, compatibility and licenses, repair failures and merge. Toolchain upgrades do not silently raise the declared Rust minimum version. App installation and rulesets are external state and must be recreated separately from Git. Decision acceptance is not activation or a successful bot-run claim. The [operating guide](../current/dependencies.md) owns actual setup, process and verification limits.

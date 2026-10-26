# Dependency maintenance

The [accepted decision](../adrs/2026-10-26-dependency-maintenance.md) selects hosted Renovate with manual review and merge. The executable policy is [`.github/renovate.json`](../../.github/renovate.json); this page describes operation and activation rather than duplicating its options.

## Update policy

New regular update branches are allowed in the first calendar week each month, Europe/Berlin. Open PRs may be updated afterwards. Up to five regular PRs can be open at once, with at most two opened per hour. New releases normally wait seven days; unavailable timestamps do not block an update indefinitely. Security fixes bypass the monthly window, age and regular PR limits at the next bot run, without a response-time guarantee.

| Update | Review unit |
| --- | --- |
| Cargo minor/patch | Rust dependency group |
| npm minor/patch | Frontend dependency group |
| Tauri crates, API and CLI | Cross-ecosystem Tauri group; major separate from minor/patch |
| Other major or pre-1.0 minor/major | Separate PR |
| Rust / Node | Separate PR per toolchain |
| GitHub Actions | Routine updates grouped; full SHA pins retained |
| Transitive packages | Monthly lockfile maintenance |
| Security fix | Separate PR when Renovate can resolve it |

Lockfile resolution can introduce transitive versions whose release age Renovate does not independently validate, especially with Cargo. An alert without an automatic fix remains maintainer work. GitHub advisories are not proof that every vulnerability is known; SHA-pinned Actions do not receive the same advisory detection as version-tagged Actions.

Rust installation follows `rust-toolchain.toml` in CI and locally. `.nvmrc` owns the application's Node version. `workspace.package.rust-version` is the declared Rust minimum, not an automatically raised toolchain pin; changing it requires an explicit compatibility decision and testing on that minimum. The pinned Renovate validation container supplies its own supported runtime. `cargo-about`'s version is maintained in `apps/desktop/package.json`; review that maintenance-tool pin when updating the tool, rather than treating it as a Cargo application dependency.

## Maintainer workflow

1. Review update PRs and upstream release notes. Check affected callers and migrations for Tauri, MIDI, toolchain and major changes. Do not bypass a failed check with a forced dependency upgrade.
2. Check out the PR, activate the pinned toolchains and run `npm --prefix apps/desktop ci`. If distribution assets changed, run `npm --prefix apps/desktop run third-party:update`. Review and commit the output on that branch. See the [notice guide](third-party.md).
3. Resolve failures, update the branch from `main` if necessary, and await all required checks on the final revision. After pushing personal fixes, maintain the branch explicitly if Renovate stops updating it to preserve those commits.
4. Merge manually and inspect CI for the exact merge commit. Routine updates do not automatically require a physical test. Changes affecting native MIDI need a targeted check and honest [verification boundaries](verification.md).

The bot automates discovery, PRs and package/lockfile edits. CI automates tests, license/source verification and builds. A maintainer or an explicitly assigned agent invokes notice generation, reviews the result, handles migrations and failures, and makes the merge decision. No workflow writes follow-up commits or approves/merges PRs.

## Activation and recovery

Repository files do not install the hosted App or configure GitHub rules. Activation order:

1. Pass local checks and the first CI run of the dependency job and all three platform builds.
2. Enable Dependency Graph and Dependabot alerts. Keep Dependabot security-update PRs disabled and do not add Dependabot version-update configuration.
3. Install the [Renovate GitHub App](https://github.com/apps/renovate) for `YNedderhoff/kit-control` only. Verify its actual repository selection and ability to read alerts.
4. Apply [`.github/rulesets/main.json`](../../.github/rulesets/main.json) through GitHub's rulesets API. It requires PRs, an up-to-date branch and four successful checks from the GitHub Actions integration, with no additional reviewer or bypass actor. Keep auto-merge disabled. For initial creation use `gh api --method POST repos/YNedderhoff/kit-control/rulesets --input .github/rulesets/main.json`; update an existing ruleset by its returned ID instead of creating a duplicate.
5. Inspect the first actual bot PR for extraction, grouping, lockfiles, source assets and successful CI before calling bot operation verified.

Activation evidence is recorded below. A valid configuration or local extraction does not establish hosted operation. To stop updates, suspend repository access for Renovate; keep required checks. Revert a faulty dependency change through a reviewed PR with passing checks.

Git contains policy, scripts, toolchain pins, lockfiles, notices and required Cargo source archives. Recreate installed packages and build tools from those inputs. Recreate the App's repository grant, alerts and ruleset separately; record their settings rather than relying on the checkout as a backup. Personal kit data is unrelated and needs its own protected backup.

## Verification and external state

Local checks on 26 October 2026 passed: 21 maintenance tests, 19 frontend tests, the frontend production build, Rust formatting/Clippy and all 65 default workspace tests (the separate virtual-CoreMIDI test was ignored). Real `third-party:update` followed by another update and check reported no further changes. All original Rust notice lines were retained; only ordering changed. An unsigned macOS application built successfully; its notices and five source archives matched the repository bytes. Local documentation links/anchors passed. Independent review found an npm substring-license bypass; the SPDX fix and real-generator regression tests passed re-review with no outstanding finding. No physical hardware was accessed.

Renovate 44.182.0's strict validator passed on its supported Node 24 runtime. A read-only local extraction found all five managers and ten representative rule evaluations confirmed routine groups, Tauri grouping, major/pre-1.0 separation and toolchain separation. Local extraction reported limitations for owner presets and missing authenticated GitHub lookup; it does not establish hosted scheduling or security-fix operation.

| External setting | Last checked state on 26 October 2026 |
| --- | --- |
| Dependabot alerts | Enabled through the API and read back successfully |
| Dependency graph | Eight manifests visible after alerts were enabled; dependency indexing still pending at that inspection |
| Dependabot automatic security PRs | Disabled, confirmed through repository settings |
| Auto-merge | Disabled |
| Renovate App | Installation not verified; available browser session requires owner sign-in |
| Required checks / `main` ruleset | Pending the first successful CI run of these changes |
| First real bot PR | Pending App activation and an eligible bot run |

Hosted installation, required-check enforcement and the first real Renovate PR must each be verified separately. CI evidence is recorded for its exact source commit after pushing; local tests alone do not establish CI success.

Read current external state rather than inferring it from this pre-activation snapshot: `gh api repos/YNedderhoff/kit-control/rules/branches/main` returns enforced rules, and `gh api repos/YNedderhoff/kit-control/vulnerability-alerts --silent` succeeds when alerts are enabled. A required-status-check rule blocks failed/missing checks; configuration readback is distinct from an actual rejected-merge test, which needs a real PR. Do not create a test PR without owner authorization.

## Tool references

Checked 26 October 2026: [Renovate configuration](https://docs.renovatebot.com/configuration-options/), [scheduling](https://docs.renovatebot.com/key-concepts/scheduling/), [release-age limits](https://docs.renovatebot.com/key-concepts/minimum-release-age/), [hosted script restrictions](https://docs.renovatebot.com/mend-hosted/faq/), and [GitHub dependency graph ecosystems](https://docs.github.com/en/code-security/reference/supply-chain-security/dependency-graph-supported-package-ecosystems).

# Maintaining third-party notices

The [distributed source and license notice](../../third-party/README.md) owns dependency-license and MPL source availability information. Original project contributions remain under the root [Apache license](../../LICENSE); distribution obligations do not establish rights to manufacturer content. See the [legal record](legal.md).

## Refresh after dependency changes

Use the [dependency workflow](dependencies.md) for bot PRs and merge requirements. Activate the project's Rust/Node toolchains as described in [development](development.md), then:

1. Review changed package versions and licenses in both lockfiles. Run `npm --prefix apps/desktop ci` to install the exact frontend tree.
2. Install the notice tool: `cargo install cargo-about --version 0.9.2 --locked --features cli`. The expected version is recorded in `apps/desktop/package.json`; the helper rejects a different version.
3. Run `npm --prefix apps/desktop run third-party:update` from the repository root.
4. Review and commit generated notices, the current `third-party/sources.json` and any new archives. Reordering the initial Rust notice makes later output independent of host ordering; upstream license texts and attributions are retained.
5. Run `npm --prefix apps/desktop run third-party:check` and the other affected checks. CI runs this read-only check before packaging. Verify built resources when changing distribution rules. A successful compiler run does not establish license compliance.

The helper uses `cargo-about --workspace --locked --fail --format json`, the existing Handlebars template, and the pinned npm license generator. It includes platform-specific Cargo dependencies without limiting collection to the host target. Selected MPL licenses determine the required Cargo archives dynamically. Downloaded and existing archives must match `Cargo.lock` checksums. Missing archives are downloaded only by `update`; `check` reports them without downloading or writing tracked files.

Both commands leave manifests and lockfiles unchanged. Generators and downloads finish and validate before any output writes. A corrupt existing archive fails rather than being silently replaced. If a later filesystem write fails, the command fails; correct the storage problem and rerun the reproducible update. Superseded archives are retained; the inventory describes the currently required versions. Pruning old archives is separate maintenance.

Frontend licenses are evaluated as complete SPDX expressions, including `AND`, `OR` and exceptions. Unknown/guessed expressions fail. MPL source delivery is automated for Cargo only: npm licenses requiring MPL fail for manual review; a dual-license expression with an accepted alternative can use that alternative. Do not approve a new source-distribution obligation without implementing its distribution path.

The notice tools are inventory aids. An unfamiliar license requires review before adding it to the accepted set in `about.toml`.

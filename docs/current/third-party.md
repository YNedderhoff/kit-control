# Maintaining third-party notices

The [distributed source and license notice](../../third-party/README.md) owns dependency-license and MPL source availability information. Original project contributions remain under the root [Apache license](../../LICENSE); distribution obligations do not establish rights to manufacturer content. See the [legal record](legal.md).

## Refresh after dependency changes

1. Review changed package versions and licenses in both lockfiles.
2. Generate Rust notices with `cargo about generate --workspace --locked about.hbs -o THIRD_PARTY_RUST.md`, using `cargo-about 0.9.2` installed with `--features cli`.
3. Run `npm run licenses` from `apps/desktop` to regenerate frontend notices with the pinned tool.
4. For changed or added MPL-covered dependencies, update their unmodified archives in `third-party/mpl-sources/` and `third-party/sources.json`; verify each archive against the new `Cargo.lock` checksum.
5. Review notice diffs and verify that the built packages still include the notices and sources. A successful compiler run does not establish license compliance.

The notice tools are inventory aids. An unfamiliar license requires review before adding it to the accepted set in `about.toml`.

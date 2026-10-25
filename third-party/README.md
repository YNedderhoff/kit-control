# Third-party source and license notices

KitControl's original files are Apache-2.0. Dependencies keep their own licenses. The complete generated Rust license texts are in [THIRD_PARTY_RUST.md](../THIRD_PARTY_RUST.md), and frontend notices are in [THIRD_PARTY_FRONTEND.txt](../THIRD_PARTY_FRONTEND.txt). The full Apache license used for the Tauri API package is also distributed in [LICENSE](../LICENSE).

## MPL-2.0 source availability

The locked Tauri dependency graph includes unmodified MPL-2.0 packages: cssparser, cssparser-macros, dtoa-short, option-ext and selectors. Their complete original source archives are included in `mpl-sources/`, under their original licenses and notices. `sources.json` records exact versions, original download URLs and SHA-256 hashes verified against `Cargo.lock`.

Recipients may obtain and modify these sources under MPL-2.0. Their rights are not restricted by KitControl's Apache license. The same archives and license notices are included in desktop app resources and CI artifact downloads. This follows the source-availability and larger-work conditions in [MPL-2.0 sections 3.1–3.3](https://www.mozilla.org/en-US/MPL/2.0/) and [Mozilla's executable-distribution FAQ](https://www.mozilla.org/en-US/MPL/2.0/FAQ/#using-and-distributing-software-under-the-license). No MPL-covered source files have been modified.

## Refresh after dependency changes

1. Review changed package versions and licenses in both lockfiles.
2. Generate Rust notices with `cargo about generate --workspace --locked about.hbs -o THIRD_PARTY_RUST.md`, using `cargo-about 0.9.2` installed with `--features cli`.
3. Run `npm run licenses` from `apps/desktop` to regenerate frontend notices with the pinned tool.
4. For changed or added MPL-covered dependencies, update their unmodified source archives and `sources.json`; verify each archive against the new `Cargo.lock` checksum.
5. Review notice diffs and verify that the built packages still include the notices and sources. A successful compiler run does not establish license compliance.

The notice tools are inventory aids. An unfamiliar license requires review before adding it to the accepted set in `about.toml`.

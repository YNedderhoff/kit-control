# Development setup and checks

Use the checkout, normally `~/git/kit-control`. Start with the [documentation index](../README.md), [contribution rules](../../CONTRIBUTING.md), and relevant project/decision records before changing behavior.

## Build and run

Prerequisites: the Rust version in `rust-toolchain.toml`, Node.js in `.nvmrc`, and the [Tauri system dependencies](https://v2.tauri.app/start/prerequisites/). The CLI does not need a graphical environment. Linux builds need ALSA development headers (`libasound2-dev` on Debian/Ubuntu) for native MIDI. Linux desktop builds also need WebKitGTK and the other Tauri dependencies; Windows desktop builds need MSVC and WebView2.

```sh
cargo test --locked
cargo build -p kitcontrol-cli --release --locked
cd apps/desktop
npm ci
npm test
npm run tauri -- dev
```

Run the CLI from the repository root:

```sh
target/release/kitcontrol ports
# Explicit no-hardware target; native target format is in docs/current/api.md.
printf '%s\n' '{"transport":"emulator"}' > /tmp/kitcontrol-target.json
target/release/kitcontrol td27 read --target /tmp/kitcontrol-target.json --kit 1 --output /tmp/kitcontrol-snapshot.json
```

To build an unsigned macOS app bundle, run `npm run tauri -- build --bundles app -- --locked` from `apps/desktop`. The bundle is written under the root `target/release/bundle/macos/`. GitHub Actions prepares Linux and Windows builds too; only actual completed runs establish CI evidence.

If Rust was installed only in the ignored local `.tooling` directory, run this from the repository root before the commands above:

```sh
export RUSTUP_HOME="$PWD/.tooling/rustup"
export CARGO_HOME="$PWD/.tooling/cargo"
export PATH="$CARGO_HOME/bin:$PATH"
```

The local Rust installation, npm dependencies and build products are not part of Git. They can be recreated from the pinned toolchains and lockfiles. Personal data needs a separate backup.

## Quality checks

Use the pinned toolchains and lockfiles. Run the checks relevant to the change; for application changes the CI workflow checks the frontend build/tests, Rust formatting, Clippy and workspace tests before packaging. See the [workflow](../../.github/workflows/checks.yml) for exact commands. The [verification record](verification.md) describes the separate macOS virtual-CoreMIDI test and its OS-service requirement.

Dependency changes also use the [maintenance workflow](dependencies.md) and [notice commands](third-party.md). Run `npm --prefix apps/desktop run test:maintenance` for the helper integration tests. The separate `dependency-checks` CI job validates Renovate and verifies distribution assets before all three builds. The validator runs in a version- and digest-pinned container so its runtime does not add a second application Node pin.

For documentation moves, check every link/anchor and plain source-path reference, preserve requirement/decision/evidence coverage, and inspect the final diff. A documentation migration is not a new software or physical-hardware verification. After an authorized push inspect CI for the exact pushed commit and report its real status.

Preserve unrelated user changes. Local toolchains and build products are reproducible; personal kit data and native backups need protected storage outside this repository.

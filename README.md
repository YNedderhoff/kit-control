# KitControl

KitControl is an early-stage drum-kit editor with a shared Rust core, CLI, and desktop application. It includes a native MIDI adapter for Roland TD-27 and a packet emulator that uses the same client. No physical hardware tests have run; hardware compatibility is not yet verified.

The application has a shared Rust core, a command-line interface, and a Tauri 2 desktop shell with a React interface. macOS, Linux, and Windows are the desktop targets. Mobile support is a later goal.

The repository name stays manufacturer-neutral. The first hardware adapter targets Roland TD-27. ATV, Alesis and other manufacturers are possible future adapters, not supported devices. The desktop application identifier is `xyz.nedderhoff.kitcontrol`; the executable is `kitcontrol`.

## Workflow

1. Choose native MIDI ports or the TD-27 packet emulator.
2. Read a supported kit snapshot and edit the supported fields offline.
3. Produce a write plan and review its changes.
4. Explicitly apply the plan to the chosen target.
5. Read back the result and check the verification report.

Offline edits never imply a device write. Native apply rereads the baseline before writing. A successful emulator check provides no evidence of physical hardware compatibility.

## Sandbox mode

Open **TD-27 device**, select **Sandbox (TD-27 emulator)**, and click **Read kit**. Edit the synthetic parameters, review a plan, then **Apply in sandbox**. Choose a sandbox scenario to try fragmented replies, timeouts, invalid packets, disconnects, or readback mismatch.

Sandbox mode uses the same TD-27 packet client as native MIDI and opens no hardware ports. Its state stays in memory for the current app session. Changing the transport or kit clears that state and its plan. A sandbox plan is marked simulated and cannot be applied to native MIDI. This is a parameter emulator, with no audio engine. It works in the packaged app; `tauri dev` is not required.

For the CLI, use an explicit `{"transport":"emulator"}` target as described in the [API contract](docs/api.md).

## Development status

This is an early development implementation, not a hardware-verified release. The initial TD-27 profile requires the user to confirm firmware **2.00** from the module. MIDI identity revision bytes do not establish the complete installed firmware version.

The first adapter reads the Kit Common block and twelve Pad blocks. It exposes kit name, tempo, tempo switch, brush switch, cross-stick switch, and twelve pad compressor switches for planned writes. This is a **partial parameter snapshot**, not a full kit, sample, or native backup. Native kit copy, tuning, and reordering remain pending. The synthetic workspace editor retains those operations as a separate development mode.

The CLI and desktop application use the same JSON dispatcher. The [development plan](docs/plan.md) records the full agreed scope; the [roadmap and implementation audit](docs/roadmap.md) identify what is implemented and still missing. The [verification record](docs/verification.md) owns executed-check evidence. See the [architecture](docs/architecture.md) and [API contract](docs/api.md) for current technical behavior.

For a new development task, start with [docs/handoff.md](docs/handoff.md). It links the project record and defines the next bounded task: plan the first physical TD-27 session, initially read-only. The full kit-editing MVP remains incomplete.

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
# Explicit no-hardware target; native target format is in docs/api.md.
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

## Scope and sources

Public emulator fixtures use invented parameter values. The TD-27 client independently implements the [documented MIDI interface](https://static.roland.com/assets/media/pdf/TD-27_MIDI_Imple_eng04_W.pdf); the repository does not redistribute Roland manuals, graphics, firmware, samples, purchased kits, or personal backups. See the [legal and provenance record](docs/legal.md), [source register](docs/sources.md), and [acknowledgements](ACKNOWLEDGEMENTS.md).

Before a physical test, follow the [hardware test protocol](docs/hardware-test-protocol.md). The [support matrix](docs/support.md) distinguishes implemented scope from verified support.

## Contributing and license

Read [CONTRIBUTING.md](CONTRIBUTING.md) before making changes. KitControl source and original documentation are licensed under [Apache License 2.0](LICENSE). See the [third-party notices and source availability](third-party/README.md) for dependencies. The working name has not undergone a trademark clearance review. This is an independent project with no claimed affiliation with or endorsement by Roland.

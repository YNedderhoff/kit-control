# KitControl

KitControl is an early-stage drum-module editor for people and coding agents, using a shared Rust core, CLI, and Tauri/React desktop application. macOS, Linux and Windows are desktop targets. The first native adapter targets Roland TD-27; physical device compatibility remains unverified. The full kit-editing MVP is not complete.

## Documentation

Start at the [documentation index](docs/README.md).

- [Build and run](docs/current/development.md#build-and-run).
- [Use the app and Sandbox](docs/current/usage.md).
- [Scope, progress and next task](docs/projects/2026-10-25-kitcontrol-editor.md).
- [Current architecture](docs/current/architecture.md), [API](docs/current/api.md), [support](docs/current/support.md) and [verification](docs/current/verification.md).
- [Hardware test procedure](docs/current/hardware-test-protocol.md).
- [Decisions](docs/adrs/README.md) and [open proposals](docs/rfcs/README.md).

The name stays manufacturer-neutral; other manufacturers and mobile support are future options, not supported devices. Sandbox mode uses the real packet client with invented parameters and no hardware ports or audio engine. Partial MIDI snapshots are not complete kit/sample backups.

## Contributions and license

Read [CONTRIBUTING.md](CONTRIBUTING.md) and [AGENTS.md](AGENTS.md). Original source/documentation is [Apache-2.0](LICENSE); dependencies retain their [notices/source availability](third-party/README.md). See the [acknowledgements](docs/current/acknowledgements.md), [source register](docs/current/sources.md) and [legal record](docs/current/legal.md).

No Roland manuals, graphics, firmware, samples, purchased kits or personal backups are distributed. KitControl is independent, with no claimed Roland affiliation or endorsement. Its working name has not undergone trademark clearance.

# Shared Rust core with Tauri and an agent CLI

- Status: accepted.
- Decision date: 2026-10-25.
- Recorded: 2026-10-26.
- Decision basis: explicit owner approval of Rust/Tauri/TypeScript/React and the subsequent complete implementation plan on 25 October 2026. Desktop targets and deferred mobile/MCP scope were included in that approval.

## Context

People need convenient kit editing on a laptop, while coding agents need dependable automation. Separate GUI and scripting implementations would duplicate validation and transfer rules. The application must support macOS and Linux without requiring the owner to use Windows; Windows is also a desktop build target.

## Decision

| Area | Decision |
| --- | --- |
| Implementation | Rust core; Tauri 2, TypeScript, and React for the desktop GUI |
| Desktop platforms | macOS, Linux, and Windows; the owner has macOS and Linux for physical testing, not Windows |
| Mobile | Consider Android and iOS in transport boundaries, layout, and touch interaction; outside the MVP |
| Automation | Full CLI in the MVP; versioned JSON contract; MCP later as another local adapter |
| Language | Initial GUI and public project documents in English; keep text suitable for later localization |

The core owns profiles and codecs, documents, editing, validation, comparison, transfer plans, and verification. It must not depend on React or Tauri. GUI, CLI, and a future MCP adapter are callers of those functions, not independent business-rule implementations.

Logical responsibilities are:

| Boundary | Responsibility |
| --- | --- |
| Device profile and protocol | Manufacturer/model identification, firmware compatibility, addresses, encodings, ranges, capabilities, and packet rules |
| Document and editing | Local typed data, validated edits/copies, history, and reference-preserving permutations |
| Transfer control | Capture, fresh-baseline checks, write plan, serial writes, readback, stop/recovery reporting |
| Native MIDI | OS port discovery, exact port selection, send/receive, connection lifetime, and device locking |
| Emulator | Virtual parameter memory and identity behind the same packet interface as hardware |
| GUI / CLI / later MCP | Input, presentation, and shared-core calls |

These are boundaries, not a requirement to create a separate crate or service for each one. The current implementation is described in [architecture.md](../current/architecture.md). Desktop MIDI uses the established `midir` library; documented SysEx support is not proof of actual device compatibility. Mobile MIDI backends must remain replaceable. Tauri's mobile capability alone does not demonstrate smartphone-to-TD-27 support.

### Core concepts

| Concept | Intended contract |
| --- | --- |
| `DeviceProfile` | Model, compatible firmware/documentation, parameter definitions, and supported operations |
| `Snapshot` | Observed data with target identity, provenance, and explicitly captured scope |
| `EditOperation` | A validated local semantic change, such as setting a parameter or copying a kit |
| `ChangePlan` | Reviewable desired changes bound to a known baseline and target |
| `ApplyReport` | Sent, read-back, uncertain, failed, and unexecuted work, with scoped evidence rather than a success-shaped fallback |

The decision requires native kit data to reach the same editing and file workflows. The [project](../projects/2026-10-25-kitcontrol-editor.md#implementation-audit) records incomplete mappings; the [current architecture](../current/architecture.md) describes the implementation.

### CLI and future MCP

The CLI must cover device selection, reading, inspection, editing, copying, reordering, comparison, planning, applying, and verification. Agent contracts include versioned JSON requests/responses, stable parameter keys, defined error and exit codes, results on stdout, diagnostics on stderr, and no interactive prompts in automation mode. Apply must name a specific plan and target; opening or editing a document must not write hardware.

MCP is not required for an agent with shell access. A later local MCP adapter must call the same functions and preserve the same write boundaries. Do not add a cloud dependency or an always-on server to provide it.

## Alternatives

C# with Avalonia was explored before the Rust choice. Separate GUI and CLI domain logic would make behavior and verification harder to keep consistent. The [independent implementation decision](2026-10-25-independent-implementation.md) owns the comparison with porting existing editors.

## Consequences

Maintain one core contract and thin adapters. Keep manufacturer-specific profiles and mobile transports replaceable without building a hypothetical universal framework. Tauri mobile capability does not establish mobile MIDI support. An accepted architecture is not evidence that all planned operations, platforms or hardware have been verified; consult the [project audit](../projects/2026-10-25-kitcontrol-editor.md#implementation-audit) and [verification record](../current/verification.md).

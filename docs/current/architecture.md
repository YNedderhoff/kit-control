# Architecture

## Shared core

The Rust core owns document validation, offline editing, comparisons, plans, and readback verification. A single serializable JSON request/response dispatcher is the application contract. CLI and Tauri invoke the same operations; neither contains a separate implementation of editing rules.

The CLI is useful for repeatable operations and integration tests. The React interface presents documents and proposed changes, then invokes the Tauri command boundary. Device transports and platform integration belong outside the pure document operations.

## Documents and plans

Synthetic documents use explicit schema versions, stable kit identities, parameter constraints, and deterministic serialization. Invalid input must produce a useful error without partially replacing a document. The synthetic model is not a representation of a Roland protocol or file format.

A plan binds an intended change to a known starting state and target. Applying a stale or mismatched plan must fail. Applying requires an explicit action, and does not follow automatically from editing, opening, comparing, or saving a file. Readback verification compares the observed state with the intended result and reports mismatches rather than treating transport completion as success.

The synthetic document simulator remains a development tool for workspace operations. The TD-27 packet emulator is a separate transport behind the same packet client used by native MIDI. It generates invented parameter values, accepts actual request/write packets, and allows fault injection. Packet-emulator outcomes cannot establish physical hardware behavior.

## TD-27 adapter

Native MIDI uses `midir` 0.11.1. The core client handles identification, bounded block reads, packet validation, planned field writes, baseline checks, and readback. The CLI and Tauri call the same dispatcher. Transport selection is explicit: native input/output ports or packet emulator.

The initial profile requires the caller to declare firmware `2.00` after checking the module. Identity replies establish device/model information, but their revision is not treated as a complete firmware version. Unknown firmware is rejected for this profile.

Snapshots contain a 52-byte Kit Common block and twelve 11-byte Pad blocks. Supported edits are `kit.name`, `kit.tempo`, `kit.tempo_enabled`, `kit.brush_enabled`, `kit.xstick_enabled`, and pad `compressor_enabled`. These blocks do not cover the full kit, instruments, samples, or native backup. Native kit copy, tuning, and reorder require additional model and protocol work.

A native apply rereads the baseline, rejects stale state, writes only planned fields, and reads back the scoped snapshot. A partial or mismatched result must remain visible. Transport completion alone is not verification. See the [API contract](api.md) for request and result handling and the [hardware test protocol](hardware-test-protocol.md) for physical tests.

The [legal record](legal.md) tracks concrete provenance and publication questions. Independent implementation of the documented interface proceeds without bundling manufacturer manuals or sound content.

## Platform boundary

The desktop target is Tauri 2 and React on macOS, Linux, and Windows. Platform builds and installer behavior need verification on each target. A successful core test on one system does not verify the desktop application or another operating system. Mobile is planned after the desktop workflow and hardware boundaries are proven.

The application identifier is `xyz.nedderhoff.kitcontrol`. Rust crate names use the `kitcontrol-` prefix; Rust has no Maven-style group namespace. The repository is `YNedderhoff/kit-control`, and its name stays generic. Roland TD-27 is the first hardware target. Other manufacturers need their own documentation, rights review and tests; no speculative manufacturer framework is implemented now.

## Decision records

See the accepted [architecture](../adrs/2026-10-25-application-architecture.md), [reuse](../adrs/2026-10-25-independent-implementation.md), and [local editing/transfer](../adrs/2026-10-25-local-edit-and-transfer-model.md) decisions for rationale and alternatives. The [editor project](../projects/2026-10-25-kitcontrol-editor.md) owns remaining implementation and acceptance work.

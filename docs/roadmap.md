# Roadmap and implementation audit

The [development plan](plan.md) defines intended requirements and milestone acceptance. This file owns the comparison with implementation, rather than redefining a smaller MVP around existing code.

Audited source: [`811c03910e860e6785ec734894da285e038d2ff6`](https://github.com/YNedderhoff/kit-control/commit/811c03910e860e6785ec734894da285e038d2ff6), inspected on 26 October 2026. The audit inspected source/tests and confirmed the exact commit's completed CI run; it did not rerun local builds/tests or access hardware. [Verification](verification.md) owns test counts, native runtime evidence, and immutable CI evidence. Refresh this audit when implementation changes.

## Milestone status

| Milestone | Actual status | Remaining acceptance work |
| --- | --- | --- |
| M0: foundation | Repository, selected stack, license/notices, provenance and contribution rules exist; complete plan is now consolidated | Name clearance and material-specific legal decisions remain open; no blanket legal approval claimed |
| M1: offline core | Synthetic document validation, editing, copying, reordering, storage, CLI and GUI are implemented/tested | Map real supported kit data into the offline document/edit/history workflow; synthetic behavior alone does not meet native MVP requirements |
| M2: Sandbox and desktop | Packet Sandbox and native adapter share the TD-27 client; GUI/CLI flows and failure paths are tested; three desktop packages build in CI | Expand Sandbox scope alongside native parameters; real kit-editor parity remains part of M4 |
| M3: physical reads | Native TD-27 identity and limited parameter reads implemented and mock-tested; macOS virtual-MIDI integration tested | Actual firmware, physical port/driver/settings, repeated reads and module-display comparison on macOS, then Linux; no physical result yet |
| M4: physical writes and MVP | Limited native field plans, endpoint/baseline checks, serial writes and readback implemented/tested against mocks | Physical reversible write/recovery evidence and all missing native kit-editing features below; not a completed MVP |
| M5: broader editor | Deferred | Broader parameters/modules, trigger/system editing, MCP, mobile; own docs/tests/evidence per profile |

## Requirement audit

IDs refer to the [MVP requirements](plan.md#3-mvp-acceptance-requirements). “Tested” below means automated/mock evidence unless physical evidence is stated explicitly.

| Requirement | Implemented and tested scope | Important remaining gap |
| --- | --- | --- |
| P01: read/open/save/compare/rename | Synthetic workspaces can load/save/diff/rename. Native GUI/CLI read partial TD-27 snapshots; CLI writes snapshot JSON; native kit name can be planned/applied | No complete native kit document; GUI native snapshot load/save and native offline-document comparison workflow are absent |
| P02: instruments and sound parameters | Synthetic instrument, tuning, muffling, volume and pan edits | No native instrument selection, tuning, muffling, pad volume or pan implementation; synthetic ranges are invented and cannot be reused as TD-27 facts |
| P03: layers and preservation | Synthetic pad/kit copy retains main/sub instruments, layer level, effects and opaque fields | Native instrument/layer blocks are not captured or copied; current captured raw-byte preservation is much narrower than whole-kit preservation |
| P04: compatible-zone copy | Shared synthetic operation supports source/destination zones; GUI offers a current-kit destination | No native pad/zone instrument-copy operation or native affected-field preview |
| P05: complete kit copy | Synthetic whole-object clone tested, including effects/opaque data | Native effects, routing, instrument and other required kit areas are missing; current snapshot cannot support a full kit copy |
| P06: swap/reorder and setlists | Synthetic permutation remaps toy setlist references; GUI has keyboard-accessible move controls | No native setlist capture/remapping, native swap/reorder, or safely preserved full-slot transfer |
| P07: undo/redo and access | Synthetic GUI history and controls tested; TD-27 GUI has labelled native controls and explicit review/apply | Native kit-edit history is absent; complete keyboard/focus/accessibility acceptance is not established for all intended workflows |
| P08: plan/apply/verify | Native target/firmware/endpoint binding, fresh full-scoped baseline, output locking, serial field writes, block/final readback and partial/conflict reporting; independent packet and real virtual-CoreMIDI tests | No physical timing/persistence/recovery evidence, multi-slot transfer, or general cancellation path; other firmware/modules unsupported |
| P09: Sandbox | Normal GUI mode and CLI emulator target use the same packet client; simulation flag/target guard prevents native apply; mode switches invalidate plans; deterministic fault scenarios | Current parameter scope only; not a firmware/audio simulation. Delayed/device-state fault coverage is bounded; see verification rather than assuming all device behavior |
| P10: human/agent parity | CLI and Tauri share the Rust JSON dispatcher for existing functions; versioned API, structured errors/results and exit codes | Full native edit/copy/reorder/history/file parity absent; synthetic cross-kit pad copy has broader CLI controls than the GUI |
| P11: readable files and state | Stable synthetic manifest/per-kit files/slot mapping, version checks and process-interruption recovery; saved-state UI separate from simulator state; native partial snapshots carry identity/scope bytes | No complete native document mapping, explicit migration implementation, or comprehensive native saved/read/verified/stale-state workflow; no power-loss or SD/sample-restore proof |
| P12: platforms and CI | Rust/UI checks, CLI and unsigned `.app`/`.deb`/NSIS artifacts pass on the audited commit; explicit native CoreMIDI virtual-port test passes on macOS | Linux/Windows GUI runtime/install behavior and physical MIDI unverified; Windows physical testing unavailable to owner; Android/iOS unimplemented |

## Code and evidence map

| Concern | Inspect |
| --- | --- |
| Shared JSON request boundary | [`crates/core/src/lib.rs`](../crates/core/src/lib.rs), [`crates/cli/src/main.rs`](../crates/cli/src/main.rs), [`apps/desktop/src-tauri/src/main.rs`](../apps/desktop/src-tauri/src/main.rs) |
| Synthetic documents/edits/files | [`model.rs`](../crates/core/src/model.rs), [`edit.rs`](../crates/core/src/edit.rs), [`storage.rs`](../crates/core/src/storage.rs), [`transfer.rs`](../crates/core/src/transfer.rs) |
| Native TD-27 scope and whitelist | [`td27.rs`](../crates/core/src/td27.rs), [`td27_connection.rs`](../crates/core/src/td27_connection.rs), [API contract](api.md) |
| Native MIDI and emulator | [`midi.rs`](../crates/core/src/midi.rs), [`td27_emulator.rs`](../crates/core/src/td27_emulator.rs) |
| GUI modes and current limitations | [`KitControl.tsx`](../apps/desktop/src/KitControl.tsx), [`Td27Panel.tsx`](../apps/desktop/src/Td27Panel.tsx), [`App.tsx`](../apps/desktop/src/App.tsx) |
| Independent expectations and CLI boundary | [`td27.rs` tests](../crates/core/tests/td27.rs), [`contracts.rs`](../crates/core/tests/contracts.rs), [`td27_commands.rs`](../crates/cli/tests/td27_commands.rs), [`commands.rs`](../crates/cli/tests/commands.rs) |
| UI contracts and CI | [`Td27Panel.test.tsx`](../apps/desktop/src/Td27Panel.test.tsx), [`App.test.tsx`](../apps/desktop/src/App.test.tsx), [`checks.yml`](../.github/workflows/checks.yml) |

The initial native profile accepts the caller-declared firmware string `2.00`. Universal identity revision bytes do not establish the installed firmware. Native snapshots comprise one Kit Common block and twelve Pad blocks; they expose name, tempo, kit switches and pad compressor switches. This is a partial parameter editor with a real native MIDI path, not a complete kit editor or a complete backup.

## Next bounded task

Plan the first physical TD-27 read-only session. First establish the actual firmware, matching documentation, native backup prerequisites, OS/driver/connection, selected MIDI endpoints and kit, independent expected display values, and stop conditions. Then run only the agreed read-only scope when the owner and device are available. Use [handoff.md](handoff.md) and the [hardware procedure](hardware-test-protocol.md).

If firmware differs from the implemented profile, assess the documented differences and required profile/tests. Do not declare `2.00`, update firmware, or remove validation to force the session through.

After physical reads are understood, select a small reversible field write and establish recovery evidence. Continue native document/parameter coverage to satisfy P01–P11. Full kit copy/swap/reorder comes only after complete required areas and native references can be preserved. While hardware is unavailable, profile/document mapping work can proceed against primary documentation and independent tests; a permanent emulator-only outcome is not acceptable.

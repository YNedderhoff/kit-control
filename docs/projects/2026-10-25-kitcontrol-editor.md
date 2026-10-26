# KitControl editor implementation

- Status: in progress.
- Start date: 2026-10-25, initial exploration and owner approval of the complete editor implementation plan.
- Documentation reorganized: 2026-10-26.
- Decision basis: owner instructions of 25–26 October 2026; implementation starts from the first physical target and preserves the complete kit-editor goal.

## Objective and scope

KitControl will be an independent open-source drum-module editor for people and coding agents. People use the desktop GUI; agents use the CLI. Both call the same Rust editing, validation, planning, transfer, and verification logic.

The product must work with physical modules. A simulator is a development and user-testing tool, not an acceptable replacement for a functioning hardware editor. Implemented native MIDI code, successful emulation, and physical device verification are distinct claims.

The first hardware target is the owner's Roland TD-27, which also contains purchased drum-tec kits. Display-observed firmware and other personal setup facts belong in the private preparation record; they are not physical MIDI test evidence. No physical test has yet been recorded. Original SD backups, user samples, and purchased kits must be preserved. A MIDI parameter snapshot must never be called a complete sample or module backup.

Support multiple Roland modules over time, each against its own documentation and firmware. Keep the project name and boundaries suitable for possible ATV, Alesis, or other manufacturer adapters. This is an expansion option, not a support commitment or a reason to build speculative abstractions now. The project is independent of home-infra.

Kit editing precedes broader module editing. The working name is KitControl; the repository is `YNedderhoff/kit-control`, executable `kitcontrol`, checkout `~/git/kit-control`, and application identifier `xyz.nedderhoff.kitcontrol`. The name was accepted on 25 October 2026; the existing remote, identifier and manufacturer-neutral expansion direction were confirmed on 26 October 2026. Name/trademark clearance remains open.

## Decision dependencies

- [Shared Rust/Tauri/CLI architecture](../adrs/2026-10-25-application-architecture.md).
- [Independent implementation and selective reuse](../adrs/2026-10-25-independent-implementation.md).
- [Local edits, transfer boundaries and Sandbox](../adrs/2026-10-25-local-edit-and-transfer-model.md).
- [Documentation lifecycle](../adrs/2026-10-26-documentation-lifecycle.md).

These records preserve accepted decisions; they do not authorize unrequested implementation, hardware writes or publication. Future open architecture questions belong in [RFCs](../rfcs/README.md).

## Research basis

The initial German and international research found recurring requests for computer editing, offline operation, kit management, bulk pad changes, and Git-friendly exports. It supplies qualitative motivation, not a market-size estimate. Adjacent models such as SPD-SX Pro are needs evidence, not TD-27 protocol evidence.

The [source register](../current/sources.md) retains the forum discussions, upstream issues, search terms, language, research date, and limits. Extend it with retrieval dates, relevant findings, and uncertainty as research continues. Do not treat inaccessible pages as proof of abandonment. Technical implementations use primary manufacturer documentation and independently specified tests, rather than forum reports as the protocol authority.

## MVP acceptance requirements

These requirements apply to real supported module data through GUI and CLI. The synthetic workspace can help develop them but cannot satisfy native acceptance by itself. The GUI may expose only some parameters conveniently at first; an operation's claimed scope must still be accurate.

| ID | Required behavior |
| --- | --- |
| P01 | Read kits from the module; open, save, inspect, compare, and rename them locally with truthful capture scope and provenance |
| P02 | Select instruments and edit documented basic sound parameters, including tuning, muffling, volume, and pan; use the actual profile's encodings and ranges |
| P03 | Preserve main/sub instruments and layer assignments through editing and copies; do not fill unknown areas with invented defaults |
| P04 | Copy instrument settings between compatible pad zones, showing exactly which settings change; exclude trigger calibration and global settings from an instrument copy |
| P05 | Copy complete kits, including all required documented kit areas such as effects and routing; preserve relevant references and captured unknown data; offer an explicitly named partial copy if scope is incomplete |
| P06 | Swap/reorder kit slots and preserve setlist targets; preview reference changes; block native reorder until reference preservation is reliable; never assume a spare slot exists |
| P07 | Undo/redo local edits; support keyboard controls, visible focus, and alternatives to drag-and-drop |
| P08 | Plan/apply/verify transfers with exact target and baseline checks, serialization, shared device locking, bounded failure handling, and honest reports of partial results |
| P09 | Provide a usable Sandbox in GUI and CLI, with original synthetic state, real packet formats, visible simulation status, reproducible faults, and rejection of Sandbox plans on native targets |
| P10 | Make the same editing and transfer capabilities available to people and agents through shared core logic and stable automation contracts |
| P11 | Store readable, stable, versioned parameter documents with useful Git diffs and a clear separation between local, saved, read, and verified module state |
| P12 | Build and test the desktop application and CLI on all three target operating systems; keep software, native MIDI, and physical support claims separate |

A complete copy or reorder must preserve everything it claims to preserve even when those fields have no dedicated GUI controls. Unknown data may be retained as read, but must not be written blindly. The current small TD-27 parameter snapshot cannot serve as a full kit image.

## User experience and Sandbox acceptance

Use familiar module terms and parameter groups in an original layout: kit list, pad/zone selection, parameter panel, and change review. Do not clone the device display or reuse Roland graphics. Optimize for laptop use, keyboard access, and later touch interaction. Keep unsupported capabilities visible and accurately scoped. Initial English strings should remain suitable for localization.

Sandbox is a normal application mode, available without running Tauri's development server. It must not open native hardware ports. Clearly mark synthetic state and clear/invalidate plans when switching target or mode. Use the same Roland packet client as the hardware path, with invented reproducible values. Keep simulation flags and endpoint binding in plan validation, not only a UI label.

The parameter emulator should cover identity, supported memory reads/writes, and faults such as delays/timeouts, absent or partial replies, bad checksums, wrong device/model IDs, fragmentation, disconnect, and readback mismatch. It does not simulate sound generation or every firmware behavior. A successful client/emulator roundtrip is insufficient as a protocol oracle; independent tests are mandatory.

## Test strategy and CI acceptance

| Level | Required independent evidence |
| --- | --- |
| Domain operations | Correct edits, compatible-zone copies, layer/unknown-field retention, undo/redo, and reference-preserving kit permutations |
| Protocol/profile | Fixed expected MIDI bytes, addresses, ranges, encodings, checksums, boundary values, reserved bytes, malformed packets, timeouts, and device confusion |
| Files | Deterministic roundtrip and diffs; corrupt/oversized input; incompatible versions; explicit migration; interrupted operations without silent data loss |
| Integration | CLI and GUI call common logic; stale/forged plans, locking, partial completion, and mismatched readback fail honestly |
| GUI | Selection, validation, focus/keyboard use, history, and review/apply state transitions |
| Native MIDI | Actual OS/backend integration, distinguished from replacing the transport with a test double |
| Physical module | Only executed checks, with exact model, firmware, OS, app commit, interface/settings, scope, expected/observed values, and recovery outcome |

Do not use production constants or encoder logic to generate expected results. Keep fixtures test-only. Public fixtures must not depend on personal kits. New modules require their own primary documents and independent expectations, not a relabeled TD-27 or TD-17 schema.

The [TD-27 MIDI implementation v2.00](https://static.roland.com/assets/media/pdf/TD-27_MIDI_Imple_eng04_W.pdf) supplies examples used as literal codec test vectors:

- DT1, first-kit snare shell-depth example: `F0 41 10 00 00 00 63 12 04 01 01 01 00 05 74 F7`.
- RQ1, first-kit snare compressor-switch example: `F0 41 10 00 00 00 63 11 04 01 41 01 00 00 00 01 38 F7`.

These are protocol test references, not instructions to send them during the first hardware session. Codec coverage of a packet does not establish a complete editable parameter profile.

GitHub Actions must test/build on macOS, Linux, and Windows: Rust formatting/lints/tests, TypeScript checking, UI tests, standalone CLI, and packaged desktop artifacts. Pin toolchains, dependencies, and actions. Distinguish simulated transport tests, native OS integration, and physical hardware checks. CI needs no private backup or samples. Do not expose fault-injection controls as an open remote service in normal distributions.

For each model/firmware/OS/function, track documented, implemented, automatically tested, and physically verified status separately. A Windows CI build is not a Windows hardware test. Hardware testing starts with the owner's TD-27 on macOS and Linux; other reports are welcome without inflating support claims.

## Milestones and acceptance

| Milestone | Required outcome | Implementation status and remaining work |
| --- | --- | --- |
| M0: preparation and foundation | Record product/reuse decisions, source and rights registers, name review limits, contribution rules, and concrete restrictions without blocking unrelated original interface work | Repository, selected stack, license/notices, provenance and contribution rules exist; complete plan is now consolidated. Name clearance and material-specific legal decisions remain open; no blanket legal approval claimed |
| M1: shared offline core | Versioned documents, validated edits/history/copies/permutations, CLI, and first GUI through common logic; synthetic fixtures establish the foundation, while P01–P07 still require a native model | Synthetic document validation, editing, copying, reordering, storage, CLI and GUI are implemented/tested. Map real supported kit data into the offline document/edit/history workflow; synthetic behavior alone does not meet native MVP requirements |
| M2: end-to-end Sandbox and desktop delivery | Read/plan/apply/verify through the packet emulator including failures; GUI/CLI shared behavior; all three desktop builds and required CI checks | Packet Sandbox and native adapter share the TD-27 client; GUI/CLI flows and failure paths are tested; three desktop packages build in CI. Expand Sandbox scope alongside native parameters; real kit-editor parity remains part of M4 |
| M3: physical TD-27 reads | Determine actual firmware and matching documentation; identify/read a bounded scope on macOS, then Linux; compare with the module and document the evidence | Native TD-27 identity and limited parameter reads implemented and mock-tested; macOS virtual-MIDI integration tested. Read-only procedure, mandatory backup gate and private report template prepared on 26 October. Complete the backup gate, physical port/driver/settings, repeated reads and module-display comparison on macOS, then Linux; no physical result yet |
| M4: physical writes and Kit Editing MVP | First prove a small reversible write; then implement and verify native P01–P12, complete kit preservation and setlist-safe copy/swap/reorder, and recovery of reserved slots | Limited native field plans, endpoint/baseline checks, serial writes and readback implemented/tested against mocks. Physical reversible write/recovery evidence and all missing native kit-editing features below; not a completed MVP |
| M5: broader editor | Extend kit parameters, effects/routing, then documented trigger/system/global settings; add modules against their own profiles and tests; MCP and mobile remain separate later work | Deferred. Broader parameters/modules, trigger/system editing, MCP, mobile; own docs/tests/evidence per profile |

A partial native field editor is useful progress toward M3/M4, not completion of M4. Hardware kit copy/swap tests must wait for complete required native coverage, even if a synthetic copy test already passes. M5's additional effect/routing editors do not defer the preservation of those areas required by MVP kit copies.

## Hardware feasibility acceptance

Plan at most two sessions of about 60–90 minutes, first macOS then Linux; backup creation and required setup are outside that budget. Follow the [hardware procedure](../current/hardware-test-protocol.md) for prerequisite checks, initial read-only work, a later reversible field write, and future full copy/swap checks on two explicitly reserved slots. It owns detailed session steps, backup/user-sample requirements, stop conditions, evidence and recovery limits. No firmware updates, factory resets, assumed empty slots, or full-sample-restore claims belong to this feasibility test.

Complete only the implemented, approved and actually executed scope. Native copy/swap acceptance must wait until all required kit areas and references are preserved. A partial parameter snapshot and the restoration of two kits do not establish complete module/sample recovery.

## Implementation audit

Audited source: [`811c03910e860e6785ec734894da285e038d2ff6`](https://github.com/YNedderhoff/kit-control/commit/811c03910e860e6785ec734894da285e038d2ff6), inspected on 26 October 2026. The audit inspected source/tests and confirmed the exact commit's completed CI run; it did not rerun local builds/tests or access hardware. [Verification](../current/verification.md) owns test counts, native runtime evidence, and immutable CI evidence. Refresh this audit when implementation changes; this documentation migration does not expand support.

### Requirement audit

IDs refer to the [MVP requirements](2026-10-25-kitcontrol-editor.md#mvp-acceptance-requirements). “Tested” below means automated/mock evidence unless physical evidence is stated explicitly.

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

### Code and evidence map

| Concern | Inspect |
| --- | --- |
| Shared JSON request boundary | [`crates/core/src/lib.rs`](../../crates/core/src/lib.rs), [`crates/cli/src/main.rs`](../../crates/cli/src/main.rs), [`apps/desktop/src-tauri/src/main.rs`](../../apps/desktop/src-tauri/src/main.rs) |
| Synthetic documents/edits/files | [`model.rs`](../../crates/core/src/model.rs), [`edit.rs`](../../crates/core/src/edit.rs), [`storage.rs`](../../crates/core/src/storage.rs), [`transfer.rs`](../../crates/core/src/transfer.rs) |
| Native TD-27 scope and whitelist | [`td27.rs`](../../crates/core/src/td27.rs), [`td27_connection.rs`](../../crates/core/src/td27_connection.rs), [API contract](../current/api.md) |
| Native MIDI and emulator | [`midi.rs`](../../crates/core/src/midi.rs), [`td27_emulator.rs`](../../crates/core/src/td27_emulator.rs) |
| GUI modes and current limitations | [`KitControl.tsx`](../../apps/desktop/src/KitControl.tsx), [`Td27Panel.tsx`](../../apps/desktop/src/Td27Panel.tsx), [`App.tsx`](../../apps/desktop/src/App.tsx) |
| Independent expectations and CLI boundary | [`td27.rs` tests](../../crates/core/tests/td27.rs), [`contracts.rs`](../../crates/core/tests/contracts.rs), [`td27_commands.rs`](../../crates/cli/tests/td27_commands.rs), [`commands.rs`](../../crates/cli/tests/commands.rs) |
| UI contracts and CI | [`Td27Panel.test.tsx`](../../apps/desktop/src/Td27Panel.test.tsx), [`App.test.tsx`](../../apps/desktop/src/App.test.tsx), [`checks.yml`](../../.github/workflows/checks.yml) |

The initial native profile accepts the caller-declared firmware string `2.00`. Universal identity revision bytes do not establish the installed firmware. Native snapshots comprise one Kit Common block and twelve Pad blocks; they expose name, tempo, kit switches and pad compressor switches. This is a partial parameter editor with a real native MIDI path, not a complete kit editor or a complete backup.

## Next bounded task

Complete the first session's [backup gate](../current/hardware-test-protocol.md#backup-gate): preserve originals, create a separately authorized current native backup including user samples, and verify local and privately uploaded copies by independent retrieval. A private Git/LFS data repository is the selected storage approach; remote creation and concrete uploads still need owner approval. No physical port discovery or parameter access may precede the completed gate.

The bounded read-only procedure and [blank private report](../current/hardware-read-report-template.md) are prepared. Keep actual device/setup facts and populated reports outside this public repository. Once backups and setup are ready, record the exact CLI revision/binary, confirm firmware/documentation and active driver support, select unique ports and the current kit, and freeze independent display/address expectations. Run only the authorized CLI identity and two partial reads, with no automatic retries or subsequent write phase. Physical results remain untested. The [verification record](../current/verification.md#read-only-session-preparation) owns the repeated mock/CLI checks for this preparation.

If firmware differs from the implemented profile, assess the documented differences and required profile/tests. Do not declare `2.00`, update firmware, or remove validation to force the session through.

After physical reads are understood, select a small reversible field write and establish recovery evidence. Continue native document/parameter coverage to satisfy P01–P11. Full kit copy/swap/reorder comes only after complete required areas and native references can be preserved. While hardware is unavailable, profile/document mapping work can proceed against primary documentation and independent tests; a permanent emulator-only outcome is not acceptable.

## Open facts and deferred choices

Personal firmware observations and available accessories are held in the private preparation record. Active driver/port/settings evidence, current complete backup and independent retrieval, physical USB/DIN behavior, and module/sample restoration remain unverified. Later write-test slots have not been reserved. Do not infer any of these from the emulator or CI.

Name/trademark clearance, material-specific legal questions, later reuse choices, further model/firmware profiles, mobile MIDI bindings, MCP transport, and automatic local commits require later work. Keep those decisions within their milestone; they are not prerequisites for unrelated documented-interface implementation.

Material-specific rights and source conditions are maintained in the [legal record](../current/legal.md), [source register](../current/sources.md), and [incorporation register](../current/acknowledgements.md). [CONTRIBUTING.md](../../CONTRIBUTING.md) owns repository placement and contribution rules. Unknown physical facts must be established with the owner, not inferred from the Sandbox or CI.

# KitControl development plan

This is the authoritative record of the agreed product scope, decisions, acceptance criteria, and development sequence. It consolidates the complete preparation and development plan, including the decisions to support real hardware, keep the project manufacturer-neutral, and provide a normal Sandbox mode. Requirements here describe the intended product, not proof that it is implemented.

The [roadmap and implementation audit](roadmap.md) compare this plan with the source. [Verification](verification.md) records checks that actually ran. [Handoff](handoff.md) gives the next bounded task. Use these repository files to continue work without access to earlier conversations.

## 1. Product and scope

KitControl will be an independent open-source drum-module editor for people and coding agents. People use the desktop GUI; agents use the CLI. Both call the same Rust editing, validation, planning, transfer, and verification logic.

The product must work with physical modules. A simulator is a development and user-testing tool, not an acceptable replacement for a functioning hardware editor. Implemented native MIDI code, successful emulation, and physical device verification are distinct claims.

The first hardware target is the owner's Roland TD-27, which also contains purchased drum-tec kits. Its installed firmware is unknown until checked on the module. No physical test has yet been recorded. Original SD backups, user samples, and purchased kits must be preserved. A MIDI parameter snapshot must never be called a complete sample or module backup.

Support multiple Roland modules over time, each against its own documentation and firmware. Keep the project name and boundaries suitable for possible ATV, Alesis, or other manufacturer adapters. This is an expansion option, not a support commitment or a reason to build speculative abstractions now. The project is independent of home-infra.

## 2. Accepted decisions

| Area | Decision |
| --- | --- |
| Name and repository | KitControl; `YNedderhoff/kit-control`; executable `kitcontrol`; working checkout `~/git/kit-control` |
| Application identifier | `xyz.nedderhoff.kitcontrol`; Rust crates use the `kitcontrol-` prefix rather than a Maven group namespace |
| Implementation | Rust core; Tauri 2, TypeScript, and React for the desktop GUI |
| Desktop platforms | macOS, Linux, and Windows; the owner has macOS and Linux for physical testing, not Windows |
| Mobile | Consider Android and iOS in transport boundaries, layout, and touch interaction; outside the MVP |
| Product sequence | Kit editing first, then a broader module editor |
| Transfer workflow | Read, edit locally, review a specific plan, explicitly apply, and read back |
| Automation | Full CLI in the MVP; versioned JSON contract; MCP later as another local adapter |
| Sandbox | Normal application mode with a TD-27 parameter emulator behind the same packet client; no audio simulation or hardware ports |
| Licensing and data | Apache-2.0 for original app contributions; personal backups, purchased kits, and samples in a separate private data repository |
| Git | People can use Git manually first; automatic local commits are a possible later app feature |
| Local operation | No cloud service or permanently running background server in the MVP |
| Reuse | Independent application with established dependencies and documented use of prior work; no ongoing fork/patch synchronization without demonstrated benefit |
| Language | Initial GUI and public project documents in English; keep text suitable for later localization |

KitControl remains a working name. Name availability and trademark rights have not been cleared. The existence of the repository does not establish those rights.

### Reuse decision and acknowledgements

A cross-platform GUI alone does not justify rebuilding an existing editor. The chosen reasons are the shared Rust GUI/CLI core, explicit plan/apply/readback contracts, readable versioned files, agent use, and a packet-level testing path shared with hardware.

| Approach | Benefit | Adaptation and maintenance cost | Decision |
| --- | --- | --- | --- |
| Full port of an existing editor | Fastest access to existing features and device knowledge | Retains model, file-format, and transfer assumptions that need assessment for offline editing, automation, and state verification; the chosen Rust stack requires a rewrite or language boundary | Do not take this route by default |
| Selective component incorporation | Can save schema, codec, or model work | C# components need a runtime/bridge or translation; translations still need independent tests; copied code/data retain provenance and license obligations | Incorporate only where the concrete saving exceeds adaptation and maintenance cost |
| Targeted new application | Direct control over shared contracts, files, and transfer behavior | More initial implementation and test work; existing functionality must not be claimed before it is rebuilt and checked | Chosen, using established libraries where they meet the requirement |

[Jon Skeet's V-Drum Explorer](../ACKNOWLEDGEMENTS.md) is a major reference for models, MIDI interfaces, schemas, firmware differences, tests, and failure cases. Its WPF GUI is platform-specific. VDrumExplorer-Cross-Platform already provides an Avalonia port, so its reported copy/paste, history, and kit-copy workflows must be considered. PulsoKit supplies another workflow and transfer reference, with a different platform scope. MIDI SYSEX-IO supplies generic GUI/CLI transfer tooling but does not replace a semantic kit editor. VDrumLib is historical prior art; its current scope and maintenance were not established.

The [acknowledgements and incorporation register](../ACKNOWLEDGEMENTS.md) owns exact project revisions, researched findings, and any actual incorporations. Inspiration, independent interface implementation, translated/copied code, copied schema/data, and ordinary package dependencies must be identified separately. A translation is still an incorporation. No author code from those researched editors is currently incorporated.

### Research basis

The initial German and international research found recurring requests for computer editing, offline operation, kit management, bulk pad changes, and Git-friendly exports. It supplies qualitative motivation, not a market-size estimate. Adjacent models such as SPD-SX Pro are needs evidence, not TD-27 protocol evidence.

The [source register](sources.md) retains the forum discussions, upstream issues, search terms, language, research date, and limits. Extend it with retrieval dates, relevant findings, and uncertainty as research continues. Do not treat inaccessible pages as proof of abandonment. Technical implementations use primary manufacturer documentation and independently specified tests, rather than forum reports as the protocol authority.

## 3. MVP acceptance requirements

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

## 4. Shared core and adapter boundaries

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

These are boundaries, not a requirement to create a separate crate or service for each one. The current implementation is described in [architecture.md](architecture.md). Desktop MIDI uses the established `midir` library; documented SysEx support is not proof of actual device compatibility. Mobile MIDI backends must remain replaceable. Tauri's mobile capability alone does not demonstrate smartphone-to-TD-27 support.

### Core concepts

| Concept | Intended contract |
| --- | --- |
| `DeviceProfile` | Model, compatible firmware/documentation, parameter definitions, and supported operations |
| `Snapshot` | Observed data with target identity, provenance, and explicitly captured scope |
| `EditOperation` | A validated local semantic change, such as setting a parameter or copying a kit |
| `ChangePlan` | Reviewable desired changes bound to a known baseline and target |
| `ApplyReport` | Sent, read-back, uncertain, failed, and unexecuted work, with scoped evidence rather than a success-shaped fallback |

The currently separate synthetic document and TD-27 snapshot types are implementation facts, not a decision to keep native kit editing disconnected from the common document workflow. Extending the real profile and its document mapping remains required.

### CLI and future MCP

The CLI must cover device selection, reading, inspection, editing, copying, reordering, comparison, planning, applying, and verification. Agent contracts include versioned JSON requests/responses, stable parameter keys, defined error and exit codes, results on stdout, diagnostics on stderr, and no interactive prompts in automation mode. Apply must name a specific plan and target; opening or editing a document must not write hardware.

MCP is not required for an agent with shell access. A later local MCP adapter must call the same functions and preserve the same write boundaries. Do not add a cloud dependency or an always-on server to provide it.

## 5. Files and device-state meaning

Use deterministic versioned JSON, a manifest, separate kit files with stable local identities, and a separate slot mapping. A reorder should change placement without rewriting every kit's contents. Stable numeric/structured values and parameter keys are authoritative; display names and formatting are not substitutes for those values.

Store timestamps and transfer reports separately from semantic kit files where practical. Distinguish at least:

1. Locally edited state.
2. State saved to files.
3. State most recently read from the device.
4. State written and read back for a recorded scope.
5. State that may have become stale since that observation.

Saving a document never proves that the module matches it. Unrecognized file versions must fail without losing data. Explicit migrations must retain originals. Preserve captured unknown bytes where safe, but do not promote them to writable, verified parameters without a profile definition.

Original SD backups, purchased kits, user samples, and personal captures belong in a separate private data repository with separate backup storage. They must not be needed by public tests or included automatically in diagnostics. A checksum establishes file integrity, not a successful module restore. Proprietary SD-backup parsing and sample transfer are outside the MVP.

## 6. Transfer and failure contract

Before writing, verify the selected device/profile/firmware and reread the affected baseline. A changed baseline must produce a conflict rather than silently overwrite another edit. Bind a native plan to its selected MIDI endpoints as well as device data. Endpoint names and universal identity do not provide a unique physical serial identity.

Serialize writes using profile-specific packet boundaries and delays. A shared lock must exclude competing KitControl processes on the same output. This cannot lock out edits on the module or unrelated software, so fresh reads still matter. Use only supported, validated field writes; TD-17 addresses must not be substituted for TD-27 addresses.

Stop on disconnect, malformed or mismatched replies, or failed readback. Report what was sent and verified, what is uncertain, and what did not run. Do not promise atomic device transactions or guaranteed automatic rollback. Resumption and recovery require another read and an assessment of the actual state. Failure injection must test this behavior instead of weakening those checks.

Native writing is an intended product function. Lack of a completed physical test must be visible in support evidence; it must not turn the project into a simulator-only demo. For later module profiles without physical verification, default to disabled experimental writes with an explicit hardware-test opt-in until that profile's conditions are documented. That cross-profile opt-in mechanism is a future requirement, not a current feature.

## 7. GUI and Sandbox

Use familiar module terms and parameter groups in an original layout: kit list, pad/zone selection, parameter panel, and change review. Do not clone the device display or reuse Roland graphics. Optimize for laptop use, keyboard access, and later touch interaction. Keep unsupported capabilities visible and accurately scoped. Initial English strings should remain suitable for localization.

Sandbox is a normal application mode, available without running Tauri's development server. It must not open native hardware ports. Clearly mark synthetic state and clear/invalidate plans when switching target or mode. Use the same Roland packet client as the hardware path, with invented reproducible values. Keep simulation flags and endpoint binding in plan validation, not only a UI label.

The parameter emulator should cover identity, supported memory reads/writes, and faults such as delays/timeouts, absent or partial replies, bad checksums, wrong device/model IDs, fragmentation, disconnect, and readback mismatch. It does not simulate sound generation or every firmware behavior. A successful client/emulator roundtrip is insufficient as a protocol oracle; independent tests are mandatory.

## 8. Repository, sources, rights, and contributions

### Layout and private data

Use an independent app repository, with Rust under `crates/`, React/Tauri under `apps/desktop/`, test-only fixtures with the relevant tests, documentation under `docs/`, and CI/contribution templates under `.github/`. Keep root license, notices, acknowledgements, contributor rules, and agent instructions discoverable. Add modules or crates when the responsibility warrants them, not as speculative infrastructure.

Public tests use original synthetic fixtures. Personal data, paid kits, native backups, samples, and private infrastructure stay out of the app repo. Do not create a separate remote data repository or change publication settings without the owner's instruction.

### Provenance and lawful use

The project must document why each relevant use is believed permissible. Keep a material/action-specific record of source, checked terms and retrieval date, rationale, limits, uncertainty, and resolution. Distinguish independent interface implementation, documentation/catalog redistribution, borrowed code/data, trademark use, and private kit data. Published MIDI documentation is not a blanket Roland permission.

The [legal record](legal.md) owns the current legal assessment and open issues. Proceed with independently written implementation of documented interface facts. Do not use a hypothetical publication concern as a blanket ban on hardware code. A concrete unresolved material restriction stops the affected action or incorporation until resolved, not automatically the whole editor. If public distribution presents a real issue, assess alternatives, including private distribution, with the owner; making a repo private does not itself establish legality.

Do not analyze/modify firmware, bypass protection, redistribute Roland manuals/graphics/firmware/samples, incorporate a complete instrument catalog without a reviewed basis, or implement proprietary SD-backup editing in the MVP. Use brands for factual compatibility descriptions in an independent design. Any Roland contact needs separate owner approval.

Original contributions use Apache-2.0. Dependencies retain their own licenses. For incorporated code or data, record original URL, exact commit, files, license, retained notices, and modifications. Preserve required copyright, license, and `NOTICE` texts in source and distributions; the app license does not relicense foreign content. These obligations do not imply a public credit on every use. [Third-party notices](../third-party/README.md) document the actual package and source-distribution arrangement.

### Contributions and AI

Adapt the supplied personal engineering/AI rules for the public project; do not copy private home-infra material or invent rules from sources not read. [CONTRIBUTING.md](../CONTRIBUTING.md) and [AGENTS.md](../AGENTS.md) own the operative rules.

Require human responsibility and maintenance decisions, meaningful tests with independent expectations, evidence separated by verification level, source/license review for new dependencies/incorporations, and transparent AI assistance. Use truthful `AI-Model:` trailers, exact model IDs only when reliably known, and no invented model identities. Contributions for other modules, test reports, issues, and PRs are welcome. That invitation does not authorize an agent to publish one on the owner's behalf.

Require applicable CI checks and human review before merging; no automatic merges. Do not claim branch protection is configured merely because contribution rules require review. Agents follow the owner's commit/push instructions for approved changes, but must obtain separate authorization for PRs, issues, messages, releases, or other external actions. Automatic Git commits by the application remain deferred.

## 9. Test strategy and CI

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

## 10. Milestones and acceptance

The milestone table below defines intended outcomes. Actual status and source evidence live only in the [roadmap](roadmap.md).

| Milestone | Completion requirement |
| --- | --- |
| M0: preparation and foundation | Record product/reuse decisions, source and rights registers, name review limits, contribution rules, and concrete restrictions without blocking unrelated original interface work |
| M1: shared offline core | Versioned documents, validated edits/history/copies/permutations, CLI, and first GUI through common logic; synthetic fixtures establish the foundation, while P01–P07 still require a native model |
| M2: end-to-end Sandbox and desktop delivery | Read/plan/apply/verify through the packet emulator including failures; GUI/CLI shared behavior; all three desktop builds and required CI checks |
| M3: physical TD-27 reads | Determine actual firmware and matching documentation; identify/read a bounded scope on macOS, then Linux; compare with the module and document the evidence |
| M4: physical writes and Kit Editing MVP | First prove a small reversible write; then implement and verify native P01–P12, complete kit preservation and setlist-safe copy/swap/reorder, and recovery of reserved slots |
| M5: broader editor | Extend kit parameters, effects/routing, then documented trigger/system/global settings; add modules against their own profiles and tests; MCP and mobile remain separate later work |

A partial native field editor is useful progress toward M3/M4, not completion of M4. Hardware kit copy/swap tests must wait for complete required native coverage, even if a synthetic copy test already passes. M5's additional effect/routing editors do not defer the preservation of those areas required by MVP kit copies.

## 11. Bounded hardware feasibility test

Budget at most two sessions of about 60–90 minutes each, first macOS then Linux. Backup creation and required setup are outside that budget. No firmware update, factory reset, or blind restore is part of the test.

The next task is to plan the first read-only session, using [hardware-test-protocol.md](hardware-test-protocol.md). Establish actual firmware rather than declaring `2.00` to bypass the current profile restriction. If firmware or documentation differs, assess the profile and independent expectations before proceeding.

Preserve original SD backups. Create an additional native backup including user samples and keep a separate protected copy before the planned write phase. Follow the [TD-27 Reference Manual](https://static.roland.com/assets/media/pdf/TD-27_Reference_eng02_W.pdf) rather than treating a MIDI export as a replacement. A checksum does not demonstrate a device restore.

The sequence is:

1. Record OS, app commit, MIDI backend/interface/driver, connection path, module settings, model, and actual firmware.
2. Perform directed identity and documented bounded reads only; compare repeated reads and module display values.
3. Only after read-only success and agreed write prerequisites, change one small reversible parameter in an owner-selected test slot, read it back, restore its original value, and read again.
4. Only after native full-kit/reference support exists, test a copy and swap between two explicitly reserved, separately secured test slots. Never assume slots are empty or disposable.
5. Restore affected slots and verify the captured scope. Stop on unexpected identity/data, incorrect mappings, unstable reads, or failed verification. Do not infer full sample/module recovery from restoring two test kits.

Record success only for executed scope. Physical fault-injection/cancellation tests require a separately assessed safe method; deterministic failure tests belong in the emulator until then. Publish only permitted, redacted evidence. The detailed procedure owns session gates and report fields; planning or documenting it does not itself authorize device writes.

## 12. Open facts and deferred choices

The personal TD-27 firmware, physical MIDI configuration, available test slots, native backup/restore evidence, and USB/DIN behavior remain unknown until measured. Do not infer them from the emulator or CI.

Name/trademark clearance, material-specific legal questions, later reuse choices, further model/firmware profiles, mobile MIDI bindings, MCP transport, and automatic local commits require later work. Keep those decisions within their milestone; they are not prerequisites for unrelated documented-interface implementation.

The original empty-workspace/no-toolchain starting state is historical. Do not repeat it as the current project state. Read the current [roadmap](roadmap.md) and [verification record](verification.md) before extending or reporting the implementation.

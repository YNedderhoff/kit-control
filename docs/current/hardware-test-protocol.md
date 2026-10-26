# Hardware test protocol

This procedure has not been executed against a physical module. It implements the bounded feasibility test in the [editor project](../projects/2026-10-25-kitcontrol-editor.md#hardware-feasibility-acceptance). The [implementation audit](../projects/2026-10-25-kitcontrol-editor.md#implementation-audit) identifies implemented scope and missing features. Start this procedure with session planning and read-only work; do not progress automatically through every phase below.

Native MIDI, Sandbox and the virtual-CoreMIDI test share protocol paths, but only physical tests can establish the connected module's behavior. The current partial snapshot is not a complete kit, module or sample backup.

## Session limits

Plan at most two sessions of about 60–90 minutes each, first macOS and then Linux. Backup creation and required installation/setup are outside that budget. Record unexecuted checks as untested when the time limit is reached. The owner has no Windows system for physical verification; Windows CI does not fill that gap.

Do not update firmware, factory-reset, or blindly restore the module to make the application work. Keep deterministic error injection in the emulator initially. Any physical disconnect/cancellation/failure test needs a safe method and recovery scope agreed before execution.

## Preconditions

1. Review the exact app revision, profile, current [API scope](api.md), and concrete restrictions in the [legal record](legal.md). A documentation plan is not an authorization to write hardware. Use the owner's actual authorization for the selected scope; do not ask them to repeat an authorization already given for that same scope.
2. Read the model and complete installed firmware version from the physical module. Record matching primary documentation. The initial reader/writer accepts declared `2.00` only; identity revision bytes do not establish full firmware. If it differs, stop parameter access and assess profile/document changes and independent tests. Do not declare `2.00` or remove the guard to force access.
3. Record OS/version, app commit, MIDI backend/interface/driver, USB or DIN connection path, exact chosen input/output names, device ID and relevant module MIDI settings. Select the intended pair explicitly. No broadcast write or automatic guess is part of the session.
4. Locate existing original SD backups without altering them. For the planned write program, create an additional native SD backup **including user samples** using the applicable [TD-27 Reference Manual](https://static.roland.com/assets/media/pdf/TD-27_Reference_eng02_W.pdf), and secure a separate protected copy. Record which files are in the private data repo, which need external backup, and which setup must be recreated. Keep backups, samples and personal captures out of the public app repo.
5. Record backup coverage and restore status honestly. A readable file or checksum proves neither full coverage nor a tested device restore. Do not perform a whole-module restore merely to check a box; agree a bounded restore procedure before claiming recovery evidence.
6. For the first read-only phase, choose one kit and independently record expected displayed values for supported parameters. Reading a kit does not make it disposable. Before phase B, explicitly reserve and secure its write-test slot. Before phase C, explicitly reserve and secure two test slots, including their relevant references; do not assume unused slots or overwrite paid kits.
7. Define expected values, address/encoding expectations, time budget, read/write scope, and stop conditions before each phase. Do not compute the expectations with the production codec under test. Physical observation and primary documentation supplement independent packet tests.

Backup creation/setup may be prepared separately; the first read-only session must record the backup situation and must not silently turn into a write session while those prerequisites remain unresolved.

## Phase A: identify and read only

1. Confirm the selected configuration and read-only scope with the owner. Use `ports`, `td27 identify`, and documented `td27 read` operations or their GUI equivalents. No apply, kit copy, swap, reorder, or raw SysEx write command belongs to this phase.
2. Check directed identity and the actual firmware/profile match. Identity contains no unique physical serial guarantee; port names and baseline equality alone do not establish that a particular unit is still attached.
3. Read the selected limited scope twice. Compare repeated bytes/values and compare supported decoded values with the module display. Preserve local evidence of discrepancies, with private names/content kept outside the app repo.
4. Record what was actually read. The existing profile captures one 52-byte Kit Common block and twelve 11-byte Pad blocks. It does not capture complete instruments, effects, routing, setlists, samples, or a whole kit.
5. End the read-only phase with pass/fail/untested results and unresolved questions. Do not infer write, persistence or recovery support from this result. A later write phase needs concrete approved scope and completed prerequisites.

## Phase B: one reversible field write

Proceed only after phase A is understood, firmware/profile matches, backup requirements are met, and an owner-selected test slot and reversible supported field are recorded. Define the original value and one small expected change independently. Review a concrete plan with target, baseline, affected bytes and recovery steps.

1. Reidentify/reread before writing and reject stale or mismatched state. Exclude other KitControl writers with the shared lock; external edits remain possible.
2. Apply only the reviewed field change through an explicit owner action. Do not substitute an example test vector, an undocumented parameter, or a complete-kit write.
3. Read back and compare with the expected value and module display. Preserve `written`, `verified`, `total`, status and message, including any reported uncertainty; transport completion is insufficient.
4. Restore the original field value through a fresh reviewed plan, then read back and compare again. If state is uncertain or unexpected, stop and assess actual state before another write, including any attempted recovery.
5. Record the successful scope and persistence limits. A restored field is not evidence of full kit, user-sample or module restoration.

## Phase C: future kit copy and swap

**Unavailable in the current native adapter.** Do not run synthetic `copy-kit`/`reorder` operations against a native target or reinterpret the partial snapshot as a full kit image.

Run this phase only after complete required kit areas and native setlist/reference preservation are implemented, independently tested, and included in the reviewed transfer plan. The GUI need not expose every preserved field, but the copy must cover them correctly.

Use exactly two explicitly reserved and secured test slots. Review source/destination contents and reference changes. Test copy and swap within those slots only, restore their original state and relevant references, and read back the implemented full scope. Stop on any unexpected identity, address, value, copy scope, reference update, or failed restoration. Restoring two kits never establishes full sample/module recovery.

## Evidence record

Keep private raw material outside the app repository. A permitted redacted report must include:

| Field | Record |
| --- | --- |
| Reproducibility | Date, app commit, commands/UI steps, expected and observed values |
| Device/profile | Model, firmware observed on the module, document revision, profile and capture scope |
| Host/connection | OS/version, MIDI backend/interface/driver, USB/DIN path, input/output names, device ID and relevant settings |
| Backup | Native backup coverage including user-sample selection, protected-copy confirmation, integrity checks, restore status and limits; private paths stay private |
| Scope and outcome | Phases attempted; each check passed/failed/untested; actual writes, readbacks, uncertainty and skipped work |
| Recovery | Original values/slot protection, restore actions actually taken, observed result, remaining uncertainty |
| Disclosure | Redaction/provenance review and permission before publishing captures, logs or a report |

Stop on unexpected identity or data, incompatible firmware, incorrect mapping, unstable reads, writes outside the plan, failed verification or failed restoration. Do not mask discrepancies with defaults or claim success from missing errors. Preserve bounded local evidence and obtain owner review before further writes when state is uncertain.

Support claims apply only to the tested model, firmware, OS, app commit, connection and function. No physical hardware report currently exists in the repository.

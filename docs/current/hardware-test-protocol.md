# Hardware test protocol

This procedure has not been executed against a physical module. It implements the bounded feasibility test in the [editor project](../projects/2026-10-25-kitcontrol-editor.md#hardware-feasibility-acceptance). The [implementation audit](../projects/2026-10-25-kitcontrol-editor.md#implementation-audit) identifies implemented scope and missing features. Start this procedure with session planning and read-only work; do not progress automatically through every phase below.

Native MIDI, Sandbox and the virtual-CoreMIDI test share protocol paths, but only physical tests can establish the connected module's behavior. The current partial snapshot is not a complete kit, module or sample backup.

## Session limits

Plan at most two sessions of about 60–90 minutes each, first macOS with direct USB and then Linux separately. Backup creation and required installation/setup are outside that budget. Record unexecuted checks as untested when the time limit is reached. The owner has no Windows system for physical verification; Windows CI does not fill that gap.

Do not update firmware, factory-reset, or blindly restore the module to make the application work. Keep deterministic error injection in the emulator initially. Any physical disconnect/cancellation/failure test needs a safe method and recovery scope agreed before execution.

## Preconditions

1. Review the exact app revision, profile, current [API scope](api.md), and concrete restrictions in the [legal record](legal.md). A documentation plan is not an authorization to write hardware. Use the owner's actual authorization for the selected scope; do not ask them to repeat an authorization already given for that same scope.
2. Read the model and complete installed firmware version from the physical module, including its build suffix. Record the observer and matching primary documentation. The initial reader/writer accepts declared `2.00` only; keep any build suffix separately in the report. Identity revision bytes do not establish full firmware. If the display differs from the profile, stop parameter access and assess documentation/profile changes and independent tests. Do not declare `2.00` or remove the guard to force access.
3. Complete the [backup gate](#backup-gate) before native MIDI discovery or opening any physical port. A local planning folder or an LFS pointer is not a completed backup.
4. Record OS/version/architecture, exact app commit and CLI binary SHA-256, MIDI backend, USB path/cable/adapters, existing USB driver mode, installed and active driver versions, and relevant MIDI settings. The USB connection uses the module's `USB COMPUTER` port. For `GENERIC`, document the OS MIDI driver; for `VENDOR`, check the active Roland driver against the current OS support information. An installer receipt does not establish that the driver is loaded, supported or functional. Record unknowns rather than silently installing drivers or changing settings.
5. After the backup gate, obtain exact input/output names with `ports` and explicitly select a unique intended pair. Read the module's MIDI device ID rather than assuming its default: display 17–32 corresponds to API/wire values 16–31. Record MIDI Tx/Rx, Soft Thru and sync settings without changing them. Close other MIDI applications; prevent external routing, loopback and concurrent edits. No broadcast request, Program Change or automatic port guess is part of the session.
6. Use the currently selected kit and record its one-based number without changing kits through MIDI. Reading a kit does not make it disposable. Before phase B, explicitly reserve and secure its write-test slot. Before phase C, explicitly reserve and secure two test slots, including their relevant references; do not assume unused slots or overwrite paid kits.
7. Before the first MIDI request, freeze the independently observed display values and documentation-derived address/encoding expectations in a private copy of the [report template](hardware-read-report-template.md). Do not derive expectations from production constants, codec, emulator or the actual snapshot. Use the selected kit's documented address, expected block sizes, model/device fields, name encoding, tempo nibbles, boolean encoding and checksum rules.

Run the existing transport-independent checks at the chosen revision before the session:

```sh
cargo test -p kitcontrol-core --test td27 --locked
cargo test -p kitcontrol-cli --test td27_commands --locked
cargo build -p kitcontrol-cli --release --locked
```

The first suite skips the native virtual-CoreMIDI test by default. That separate test includes writes to virtual endpoints; do not describe it as a read-only physical test or enable it implicitly. The CLI suite uses synthetic emulator state. Follow the [development guide](development.md) for the pinned toolchain. Keep test, build, physical and recovery evidence separate.

## Backup gate

Backup preparation is separate from phase A and requires its own concrete authorization before any SD write or upload. The module's documented native backup path is SD, not USB file transfer; the MIDI snapshot cannot replace it. Use the applicable [Reference Manual, pages 48–49](https://static.roland.com/assets/media/pdf/TD-27_Reference_eng02_W.pdf).

1. Inventory existing SD backups, purchased kit originals and original samples without altering them. Preserve relative paths, file sizes and SHA-256 hashes in a local archive outside the app checkout. Do this before a new SD save. Do not format, delete, renumber samples or overwrite any original. Stop on copy/read errors or an inventory/hash mismatch.
2. Prepare an additional native `BACKUP ALL` of the current module state with `With User Sample` selected and an explicitly free backup bank. Check available space and record the completion indication and sample inclusion. Purchase originals do not capture later edits or reordered kits. If there is no safe free bank or enough storage, stop instead of replacing an old backup.
3. Copy the new backup into the private archive and inventory it as well. Keep the original SD files and the local archive. Hashes establish byte integrity, not native backup coverage or a successful module restore. Record current sample count and any coverage uncertainty separately.
4. Prepare a separate private data repository: plain Git for readable JSON, inventory and reports; Git LFS for binary backups and sample originals. Keep source files unmodified and timestamped captures/reports separate from semantic exports. Check concrete file sizes, account LFS storage/transfer availability and private visibility before an owner-reviewed upload. Creating a remote and uploading files require separate owner approval. No public app fixtures, CI artifacts, automatic diagnostics or public links may contain these files.
5. From the approved private remote, use a fresh clone with a fresh LFS storage directory, without local alternates or a shared LFS cache. Fetch all archived LFS objects, materialize the inventoried files at their recorded revisions and compare sizes and hashes against the local archive. Missing objects, pointer-only files or mismatches fail the gate. A GitHub ZIP or an ordinary Git clone alone does not establish complete LFS retrieval. See the [GitHub repository backup procedure](https://docs.github.com/en/repositories/archiving-a-github-repository/backing-up-a-repository).
6. Retain the SD originals, local archive and verified remote copy without automatic deletion or destructive synchronization. Record source/archive/data-repository locations privately. Git holds app sources and, in the private repo, text records plus LFS references; the actual binary content needs the archive and available LFS objects. Recreate toolchains, drivers, Git/LFS tools and MIDI setup separately. Device restore remains **untested**; do not perform a full restore just to validate this gate.

The gate passes only with preserved originals, a completed current backup including user samples, a checked local archive, and independently retrieved remote files matching the inventory. Otherwise phase A stays blocked. A later native restore procedure needs separately agreed scope.

## Phase A: identify and read only

1. Check that the backup gate passed and the owner authorized the selected read-only session. Use the CLI command sequence below, one command at a time, checking its exit code and JSON response before proceeding. Create no plans and run no apply, kit copy, swap, reorder, raw SysEx write or restore operation.
2. Check directed identity and the display-confirmed firmware/profile match. Identity contains no unique physical serial guarantee; port names and baseline equality alone do not establish that a particular unit is still attached. Allowed outbound messages are directed Identity Request and Roland RQ1 only. Inbound DT1 is the documented read reply; outbound DT1 is a write and is excluded. This is an operational scope, not a new software-enforced read-only transport mode.
3. Read the selected limited scope twice into distinct new private files. Do not change kits/parameters, play pads, or send MIDI control/clock messages between observations. Each `read` also identifies the target before requesting its blocks. A successful complete sequence sends three identity requests and 26 RQ1 requests; do not claim observed packet counts without a capture.
4. Check `simulated: false`, the expected identity, kit and selected endpoints, and exactly 13 complete blocks: one 52-byte Kit Common block and twelve 11-byte Pad blocks, **184 bytes per snapshot**. Match addresses/lengths to the pre-recorded independent expectations, then compare all raw block bytes and decoded values between the two captures. The snapshots do not capture complete instruments, effects, routing, setlists, samples, or a whole kit.
5. Compare the 17 decoded values with the frozen display observations: name, stored kit tempo, kit-tempo/brush/Xstick switches and twelve pad compressor switches. Use the kit-tempo menu rather than the global click tempo, and the pad compressor rather than the master compressor. Mark hidden or inaccessible values untested; do not toggle parameters to expose them. A matching unknown byte is only stable across reads, not semantically verified. Recheck visible values after the reads without editing them.
6. End with passed/failed/untested results and unresolved questions. A complete display comparison requires evidence for every decoded value; otherwise report the verified subset. Stop without automatic retries on a discrepancy or error. Do not infer write, persistence, full-kit preservation or recovery support.

The following are **later manual steps**, not a script to run before the gates pass. Resolve `cli`, `session` and `kit` to the reviewed absolute CLI path, a new private session directory outside the app repository, and the recorded current kit number. Do not paste placeholder values into an unattended run.

```sh
"$cli" ports
"$cli" td27 identify --target "$session/target.json"
"$cli" td27 read --target "$session/target.json" --kit "$kit" --output "$session/read-a.json"
"$cli" td27 read --target "$session/target.json" --kit "$kit" --output "$session/read-b.json"
```

Construct `target.json` using the [native target contract](api.md#td-27-midi-operations) after observing firmware, device ID and unique ports. Use `transport: "native"`; enter the exact firmware string accepted by the matching profile and retain the display build separately. Retain stdout, stderr and each exit code privately without overwriting previous evidence. The output snapshot files contain the result directly; stdout uses the API response envelope. A failed read may have contacted hardware without producing a snapshot file, so record the failure rather than inferring no activity.

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

Start with a private copy of the [read-only report template](hardware-read-report-template.md). Keep populated reports, targets, snapshots and raw material outside the app repository. Publication needs owner approval of the concrete redacted report. A permitted redacted report must include:

| Field | Record |
| --- | --- |
| Reproducibility | Date, app commit, commands/UI steps, expected and observed values |
| Device/profile | Model, firmware observed on the module, document revision, profile and capture scope |
| Host/connection | OS/version, MIDI backend/interface/driver, USB/DIN path, input/output names, device ID and relevant settings |
| Backup | Native backup coverage including user-sample selection, protected-copy confirmation, integrity checks, restore status and limits; private paths stay private |
| Scope and outcome | Phases attempted; each check passed/failed/untested; actual writes, readbacks, uncertainty and skipped work |
| Recovery | Original values/slot protection, restore actions actually taken, observed result, remaining uncertainty |
| Disclosure | Redaction/provenance review and permission before publishing captures, logs or a report |

Stop before opening physical ports if the backup gate, firmware, driver support, target selection or independent expectations remain unresolved. During a session stop on ambiguous/lost ports, unexpected identity or data, incompatible firmware, timeout, bad checksums, unexpected addresses/lengths, incomplete or unstable reads, display mismatch, unexpected state changes, outbound writes, failed verification or failed restoration. Stop at the time limit too. Do not mask discrepancies with defaults, retry automatically, repair settings or claim success from missing errors. Preserve bounded local evidence and obtain owner review before any further operation when state is uncertain.

Support claims apply only to the tested model, firmware, OS, app commit, connection and function. No physical hardware report currently exists in the repository.

# API and CLI contract

## Version 1

The Tauri `dispatch` command and `kitcontrol request <file>` use the same Rust dispatcher. Use `kitcontrol request -` for JSON on standard input. Requests may include `"api_version": 1`; omission means version 1. Other versions, unknown actions and unknown fields are rejected. Input and output JSON files are limited to 16 MiB.

Every response has `api_version` and `ok`, then either `result` or `error: {code, message}`. `ok` means the request was executed without an API error. For an apply result, **only `status: "verified"` means the intended state within the reported scope was read back**. A conflict, failed write or partial result can have `ok: true` and still has a nonzero CLI exit code.

| Exit code | Meaning |
| --- | --- |
| 0 | Request succeeded, transfer verified, or verification matched |
| 2 | Invalid input, unsupported format or invalid operation |
| 3 | I/O or MIDI transport error, busy device/workspace, or interrupted save |
| 4 | Apply conflict, failed transfer or readback mismatch |
| 5 | Explicit verification did not match |
| 6 | Hardware or unsupported profile is locked |

JSON results go to standard output; error diagnostics go to standard error. `--help`, `--version` and CLI argument syntax follow clap's text interface. Automation should use `request` when it requires JSON errors for malformed requests.

## Operations

| Request action | Input | Result |
| --- | --- | --- |
| `demo` | None | Original synthetic document, snapshot and profile |
| `devices` | None | Development profiles; use `midi_ports` for native ports |
| `validate` | `document` | Validation result |
| `edit` | `document`, `operation` | New validated document; input unchanged |
| `diff` | `before`, `after` | Sorted changes with JSON Pointer paths |
| `plan` | `baseline` snapshot, `desired` document | Plan bound to identity and both document hashes |
| `simulate_apply` | `device` snapshot, `plan`, optional `fault` | Apply report and actual simulated snapshot |
| `verify` | `device` snapshot, `expected` document | Match flag, hashes and differences |
| `load` / `save` | Workspace `path`, plus `document` for save | Document / saved file count |
| `recover` | Workspace `path` | Completion of a pending save |

Edit kinds are `rename_kit`, `set_parameter`, `copy_pad`, `copy_kit` and `reorder_kits`. Kit slots and setlist entries are one-based; pad names and kit IDs are stable strings. A kit copy needs a fresh `new_kit_id`; the overwritten kit remains in the local library. Reordering needs an exact permutation of occupied kit IDs and updates setlist references to retain kit identity.

The synthetic profile has four example instruments. Numeric ranges are invented for this profile: tuning -100–100, muffling 0–10, volume 0–100, pan -30–30 and layer level 0–100. These are not Roland parameter definitions. Main/sub-instrument assignments, layer level, effects and opaque bytes are retained by the relevant whole-object copies. There is no trigger-calibration model or audio engine.

Plans contain the complete baseline and desired documents, identity, hashes and derived differences. Applying revalidates all of them. The initial implementation conservatively rejects any baseline document change, including changes outside the edited field. The hashes detect accidental changes; they are not signatures or an authorization boundary.

## TD-27 MIDI operations

These actions use the same dispatcher from `kitcontrol request` and Tauri. They are distinct from the synthetic workspace actions above.

| Request action | Input | Result |
| --- | --- | --- |
| `midi_ports` | None | Native MIDI input and output ports |
| `td27_identify` | `target` | Directed TD-27 identity response; no parameter access |
| `td27_read` | `target`, one-based `kit` (1–100) | Partial TD-27 snapshot |
| `td27_plan` | `baseline` snapshot, `changes` object | Validated field write plan; no MIDI writes |
| `td27_apply` | `target`, reviewed `plan` | Report with status, write/readback counts, message, and optional resulting snapshot |

Select a target explicitly. Native targets provide `transport: "native"`, `input`, `output`, `device_id`, and `firmware`. Use the exact unique port-name strings returned in `inputs` and `outputs` by `midi_ports`. Native snapshots retain these names in `endpoint`; a plan cannot be applied using different ports. MIDI identity exposes no unique serial number, so reused port names do not prove the same physical unit is still attached. Emulator targets provide `transport: "emulator"`, optional `state`, and optional `device_id`, `firmware`, and `fault`. Emulator defaults are device ID `16`, firmware `"2.00"`, and fault `"none"`. Both transports use the same packet client.

The API device ID is the wire value 16–31, corresponding to the module display 17–32. The initial profile requires exact declared firmware `"2.00"`. Check the module before declaring it; the identity reply's `software_revision` is retained but does not establish full firmware version.

A no-hardware read request is:

```json
{"action":"td27_read","target":{"transport":"emulator"},"kit":1}
```

Snapshots contain `format_version`, `identity`, `kit`, `simulated`, raw scoped `blocks`, and decoded `parameters`. The scope is one 52-byte Kit Common block plus twelve 11-byte Pad blocks. The snapshot is not a complete kit, instrument, sample, or backup image. Do not manufacture a native baseline from an emulator snapshot: apply checks the transport type as well as device ID, firmware, and baseline state.

Pass the returned snapshot as `baseline` to `td27_plan`. `changes` is an object keyed by supported parameter names, for example:

```json
{"kit.name":"Practice","kit.tempo":120,"pad.snare.compressor_enabled":true}
```

| Parameter | Accepted value |
| --- | --- |
| `kit.name` | 1–12 printable ASCII characters |
| `kit.tempo` | Integer 20–260 |
| `kit.tempo_enabled` | Boolean |
| `kit.brush_enabled` | Boolean |
| `kit.xstick_enabled` | Boolean |
| `pad.<pad>.compressor_enabled` | Boolean |

Pad names are `kick`, `snare`, `tom1`, `tom2`, `tom3`, `hihat`, `crash1`, `crash2`, `ride`, `aux1`, `aux2`, and `aux3`. Unsupported fields fail explicitly. Native tuning, kit copying, and reordering are not implemented by this profile.

Plans retain the baseline, baseline hash, requested changes, and derived writes. Apply recomputes and validates the plan, identifies and rereads the selected kit, rejects stale baselines, then writes only changed fields. It reads back each affected block and the final scoped snapshot. Status `verified` establishes readback only for that scope. `conflict` means the baseline or target did not match; `partial` means writes or verification did not complete and state may need a fresh read. Counts describe planned fields, not whole kits or durable storage guarantees.

The report's optional `snapshot` can be reused as emulator `state` for the next request. If it is absent after an interrupted operation, do not assume the previous snapshot still represents the device. Native apply has no automatic rollback; read and assess the module before further writes. Closing the GUI loses unsaved in-memory emulator state. Preserve snapshots explicitly when a repeatable emulator session is needed.

## TD-27 CLI workflow

The named commands use the same request actions:

```sh
kitcontrol ports
kitcontrol td27 identify --target target.json
kitcontrol td27 read --target target.json --kit 1 --output snapshot.json
kitcontrol td27 plan --baseline snapshot.json --changes changes.json --output plan.json
kitcontrol td27 apply --target target.json --plan plan.json
```

An emulator `target.json` contains `{"transport":"emulator"}`. A native target contains, for example, `{"transport":"native","input":"<exact input name>","output":"<exact output name>","device_id":16,"firmware":"2.00"}`. Replace the names using `ports`; verify firmware on the module. `changes.json` contains the parameter/value map shown above. Snapshot and plan destinations must be new files. Apply returns its report on stdout; it does not overwrite the target or plan file.

To continue an emulator session, put the returned snapshot into the target's `state` field. Native sessions always reread the hardware. The fault names for the packet emulator are `none`, `timeout`, `bad_checksum`, `wrong_device`, `wrong_model`, `disconnect`, `readback_mismatch`, and `fragmented`. Packet faults affect reads; disconnect and mismatch are injected on writes. `written` counts successfully sent field messages. `verified` counts fields matching the final snapshot when available; a failed final read clears that count. A partial report never establishes final device state merely from its counts.

## Synthetic CLI workflow

1. Create the original workspace and explicit simulated device state:

   ```sh
   kitcontrol demo --output private-data/original --device private-data/device.json
   ```

2. Write a local operation file, for example `private-data/tune.json`:

   ```json
   {"kind":"set_parameter","kit_id":"kit-1","pad":"snare.head","parameter":"tuning","value":5}
   ```

3. Edit and review without changing the device:

   ```sh
   kitcontrol edit private-data/original --operation private-data/tune.json --output private-data/edited
   kitcontrol diff private-data/original private-data/edited
   kitcontrol plan --baseline private-data/device.json --workspace private-data/edited --output private-data/plan.json
   ```

4. Apply the reviewed plan and verify:

   ```sh
   kitcontrol apply --device private-data/device.json --plan private-data/plan.json
   kitcontrol verify --device private-data/device.json --workspace private-data/edited
   ```

`copy-pad`, `copy-kit`, `reorder`, `validate`, `inspect` and `recover` are also available; use their `--help` output for arguments. New plan and device files use atomic no-clobber creation. A reviewed plan cannot silently replace an existing plan. Workspaces and device state files use process locks during writes.

`simulate_apply` is stateless: the caller supplies a snapshot and receives the resulting snapshot. GUI simulator state lives in that window; the CLI `apply` command persists its explicit device JSON file. They share the transfer logic, not a background device service. Closing the GUI discards its unsaved simulator state.

Faults are `none`, `disconnect` (before the single synthetic document write), and `readback_mismatch` (after writing). They do not model timed MIDI packets, partial SysEx blocks, cancellation or actual device persistence.

## Workspace files and recovery

The manifest contains format/profile/document identity, the kit inventory, slot order and setlists. Individual kits live in `kits/<stable-id>.json`. Saving retains unoccupied library kits and does not delete old kit files. There is one authoritative location for each persisted fact; the pending transaction file is temporary recovery state.

A save writes `.kitcontrol-pending.json`, replaces kit files and the manifest, then removes that marker. An interrupted save refuses a normal load; `kitcontrol recover <workspace>` explicitly completes the pending state. Unsupported versions, unsafe identifiers, symbolic links at managed paths and foreign documents are rejected.

The tested recovery scope is **process interruption**. Atomic file replacement, file flushes and Unix directory flushes improve ordering, but sudden power loss and full storage failure have not been tested. Newly created parent directories are not durably flushed, and Windows has no portable directory flush here. Do not treat workspace files as a replacement for external backups.

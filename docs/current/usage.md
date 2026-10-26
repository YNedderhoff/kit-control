# Using KitControl

KitControl exposes a native TD-27 connection, the packet Sandbox, and a separate synthetic offline workspace. The [support matrix](support.md) and [API contract](api.md) define available capabilities; the complete kit-editor goal remains in the [project](../projects/2026-10-25-kitcontrol-editor.md).

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

For the CLI, use an explicit `{"transport":"emulator"}` target as described in the [API contract](api.md).

## Physical module work

Follow the [hardware test procedure](hardware-test-protocol.md) and verify the installed firmware against the profile. Read-only, write, and recovery results are separate. Saving parameter files does not create a native SD backup or transfer user samples.

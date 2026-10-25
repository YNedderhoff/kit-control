# Hardware test protocol

This procedure has not been executed against a physical module. Native MIDI and the packet emulator share the TD-27 client, but emulator success does not verify physical writes or recovery. The partial parameter snapshot is not a complete backup.

## Preconditions

1. Review the exact adapter revision, documented interface scope, and any concrete concern in the [legal record](legal.md).
2. Obtain the device owner's explicit authorization for the planned reads and writes.
3. Record device model, firmware shown by the device, operating system, MIDI interface, driver, connection path, and application commit. The initial profile requires exact firmware `2.00`; declare it only after checking the module. Identity revision bytes are insufficient.
4. Identify the authoritative backup mechanism and where its files are kept. Record which files are in Git, which need a separate backup, and which settings must be recreated.
5. Create an owner-controlled backup. Test the relevant restore path before claiming recovery is verified. Keep user backups and device captures outside the repository.
6. Select a disposable test kit and a small bounded parameter change. Define the expected state independently of implementation constants.

## Execution

1. Start with read-only discovery. Confirm the device identity and firmware compatibility before accessing data.
2. Read the selected state twice and compare it. Resolve instability before continuing.
3. Generate a plan with explicit target, starting state, requested changes, and expected readback. Have the owner review it.
4. Apply only after an explicit approval action. Log progress and errors without publishing personal data.
5. Read back the affected state and compare it with independently specified expected values. A successful send is not verification.
6. Restore the prior state, read it back, and record the outcome.
7. Test timeout, cancellation, stale-plan rejection, disconnect, partial completion, and mismatch behavior using a safe fault-injection method agreed before execution. Stop when state is uncertain.

## Evidence and stop conditions

Record expected and observed values, pass/fail/untested status, logs, recovery outcome, and any limits. Never infer success from missing errors. Stop on unexpected identity, unrecognized firmware, writes outside the plan, unstable reads, or failed restore. Preserve enough local evidence to diagnose partial completion, then seek owner review before further writes.

Publish only a redacted result with permission. Support claims apply solely to the tested model, firmware, adapter revision, and connection path until additional evidence exists.

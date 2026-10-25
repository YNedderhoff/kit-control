# Roadmap

Milestones describe scope, not release promises. Record actual checks against a commit in [verification.md](verification.md).

| Milestone | Scope | Status |
| --- | --- | --- |
| M0: foundation | License, provenance, source register, workspace | In place; concrete publication and name questions tracked in [legal.md](legal.md) |
| M1: offline core and CLI | Synthetic validation, editing, copy/reorder, diff, plan/apply/verify | Implemented development mode; does not establish native kit operations |
| M2: desktop workflow | Tauri 2 and React, same dispatcher, explicit plan review | Synthetic and TD-27 emulator workflows implemented; macOS native IPC checked |
| M3: device adapter | Native MIDI ports, TD-27 identity and scoped reads, same-client packet emulator | Initial firmware-2.00 profile implemented and packet-tested; physical tests pending |
| M4: device writes | Reviewed field plans, stale-state rejection, readback | Initial limited fields implemented and packet-tested; physical write and recovery evidence pending |
| Next | Native tuning, kit copy/reorder, broader parameters | Not implemented by the partial snapshot |
| Later | Mobile, other manufacturers, MCP, release distribution | No committed schedule |

## Acceptance gates

Offline operations need independent expected values, invalid-input checks, preserved input on failure, stale-plan rejection, and verification mismatch tests through public integrations.

Desktop changes need an actual build and working native IPC, with evidence for each claimed operating system. A frontend bundle alone does not verify the desktop application.

TD-27 packet checks need independently specified protocol expectations, invalid checksum/identity/model handling, fragmentation, timeout/disconnect behavior, stale baselines, partial writes, and readback mismatch. Use invented parameter values in public tests. Documented interface code and emulator code must stay separate enough for tests to detect client errors.

Physical support needs the [hardware test protocol](hardware-test-protocol.md), exact model/firmware and connection evidence, and a tested restore path before claiming recovery. No emulator, build, or CI result establishes physical compatibility. GitHub Actions configuration is source code; only completed workflow runs establish CI success.

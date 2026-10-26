# Start here for the next task

Use this file from a new task opened in `~/git/kit-control`. The repository is the project record; no previous conversation is required.

## Read order

1. [AGENTS.md](../AGENTS.md) and [CONTRIBUTING.md](../CONTRIBUTING.md): operative engineering, attribution and publication rules.
2. [Development plan](plan.md): full agreed product scope and acceptance requirements.
3. [Roadmap and implementation audit](roadmap.md): actual progress and gaps, with source evidence.
4. [Verification](verification.md), [support](support.md), [architecture](architecture.md), and [API](api.md): evidence and existing contracts.
5. [Hardware test protocol](hardware-test-protocol.md), [legal record](legal.md), and [source register](sources.md): session prerequisites and boundaries.

Inspect current Git status/history before changing anything. The roadmap names its audited implementation commit; newer commits need their own assessment. Do not overwrite user changes.

## Next task

Plan the first bounded physical TD-27 session, initially read-only. A planning pass is appropriate because actual firmware, hardware connection, backups and expected values are still unknown. This handoff does not authorize an automatic write session.

The installed firmware must be read from the module. The initial code accepts declared `2.00` only; identity revision bytes and the emulator are not substitutes for that observation. If it differs, review matching documentation and profile changes before proceeding. Do not bypass the check or update firmware to match the software.

Reuse existing tools and authorization for the selected scope. Establish missing physical facts with the owner rather than asking them to repeat already documented architecture choices. Keep original SD backups and user samples protected. All private kit/capture data stays outside the public app repository. Follow the hardware procedure's stop conditions and record results with exact scope.

The real MIDI adapter and Sandbox exist; the MVP is not complete. Native tuning, instrument/pad copies, full kit copies, swap/reorder/setlist preservation, full native documents and history remain pending. Those limits are detailed in the roadmap. Do not reinterpret synthetic operations or CI success as real hardware verification.

## Suggested opening request

> Read the development plan, roadmap, verification record and hardware-test protocol. Plan the first limited TD-27 hardware session with me, initially read-only. Check actual firmware and matching documentation, native backup prerequisites, the MIDI/OS configuration, expected module values, and stop conditions. Keep implementation, automatic/mock tests and physical results separate. Do not start a write/copy/reorder session from this request.

If the module is unavailable, continue only independent documentation/profile/test work within the requested scope. A later write session needs its own concrete reviewed target, parameter, baseline and recovery plan; approval of a plan is not proof of a hardware result.

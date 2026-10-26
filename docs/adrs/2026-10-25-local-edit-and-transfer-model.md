# Local editing, explicit transfer plans and isolated simulation

- Status: accepted.
- Decision date: 2026-10-25.
- Recorded: 2026-10-26.
- Decision basis: the complete implementation plan approved on 25 October 2026. The owner reaffirmed real hardware support and the usable mock/Sandbox path on 26 October 2026; those requirements do not replace physical verification.

## Context

A saved file, a captured parameter range, a completed send, and a verified physical module state are different things. Personal TD-27 backups include purchased kits and user samples that a parameter editor cannot safely treat as replaceable public fixtures.

## Decision

Edit locally, review a specific target/baseline-bound change plan, apply explicitly, and read back. Provide a normal parameter Sandbox behind the same packet client, without opening native ports or simulating audio. Mark simulation in snapshots and plan validation so a Sandbox plan cannot accidentally be applied to native MIDI.

Original app contributions use Apache-2.0. Personal backups, purchased kits and samples belong in a separate private data repository and separate protected backup storage. Git can be used manually first; automatic local commits are deferred. Do not require a cloud service or permanently running background server for the MVP.

### Files and device state

Use deterministic versioned JSON, a manifest, separate kit files with stable local identities, and a separate slot mapping. A reorder should change placement without rewriting every kit's contents. Stable numeric/structured values and parameter keys are authoritative; display names and formatting are not substitutes for those values.

Store timestamps and transfer reports separately from semantic kit files where practical. Distinguish at least:

1. Locally edited state.
2. State saved to files.
3. State most recently read from the device.
4. State written and read back for a recorded scope.
5. State that may have become stale since that observation.

Saving a document never proves that the module matches it. Unrecognized file versions must fail without losing data. Explicit migrations must retain originals. Preserve captured unknown bytes where safe, but do not promote them to writable, verified parameters without a profile definition.

Original SD backups, purchased kits, user samples, and personal captures belong in a separate private data repository with separate backup storage. They must not be needed by public tests or included automatically in diagnostics. A checksum establishes file integrity, not a successful module restore. Proprietary SD-backup parsing and sample transfer are outside the MVP.

### Transfer and failures

Before writing, verify the selected device/profile/firmware and reread the affected baseline. A changed baseline must produce a conflict rather than silently overwrite another edit. Bind a native plan to its selected MIDI endpoints as well as device data. Endpoint names and universal identity do not provide a unique physical serial identity.

Serialize writes using profile-specific packet boundaries and delays. A shared lock must exclude competing KitControl processes on the same output. This cannot lock out edits on the module or unrelated software, so fresh reads still matter. Use only supported, validated field writes; TD-17 addresses must not be substituted for TD-27 addresses.

Stop on disconnect, malformed or mismatched replies, or failed readback. Report what was sent and verified, what is uncertain, and what did not run. Do not promise atomic device transactions or guaranteed automatic rollback. Resumption and recovery require another read and an assessment of the actual state. Failure injection must test this behavior instead of weakening those checks.

Native writing is an intended product function. Lack of a completed physical test must be visible in support evidence; it must not turn the project into a simulator-only demo. For later module profiles without physical verification, default to disabled experimental writes with an explicit hardware-test opt-in until that profile's conditions are documented. That cross-profile opt-in mechanism is a future requirement, not a current feature.

## Alternatives and consequences

Immediate writes during editing would remove the review boundary. Treating exports as full backups would hide missing sample and kit areas. A simulator-only product would fail the hardware requirement. A separate protocol implementation for the Sandbox would weaken its integration value; independent test expectations are still necessary to detect shared client/emulator bugs.

Keep session scope, state freshness, partial results and recovery limits explicit. This decision specifies the required behavior; the [current API](../current/api.md) and [project audit](../projects/2026-10-25-kitcontrol-editor.md#implementation-audit) identify what is actually implemented. The [hardware procedure](../current/hardware-test-protocol.md) governs physical evidence and prerequisites.

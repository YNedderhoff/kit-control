# Independent implementation with selective reuse

- Status: accepted.
- Decision date: 2026-10-25.
- Recorded: 2026-10-26.
- Decision basis: the owner's request to compare porting/reuse costs, acknowledge researched projects, and implement the complete plan that selected a targeted new application on 25 October 2026.

## Context

A cross-platform GUI alone does not justify rebuilding an existing editor. The required shared Rust GUI/CLI contract, reviewable transfers, versioned files, and independent packet testing must be evaluated alongside existing functionality and adaptation costs.

## Decision and alternatives

A cross-platform GUI alone does not justify rebuilding an existing editor. The chosen reasons are the shared Rust GUI/CLI core, explicit plan/apply/readback contracts, readable versioned files, agent use, and a packet-level testing path shared with hardware.

| Approach | Benefit | Adaptation and maintenance cost | Decision |
| --- | --- | --- | --- |
| Full port of an existing editor | Fastest access to existing features and device knowledge | Retains model, file-format, and transfer assumptions that need assessment for offline editing, automation, and state verification; the chosen Rust stack requires a rewrite or language boundary | Do not take this route by default |
| Selective component incorporation | Can save schema, codec, or model work | C# components need a runtime/bridge or translation; translations still need independent tests; copied code/data retain provenance and license obligations | Incorporate only where the concrete saving exceeds adaptation and maintenance cost |
| Targeted new application | Direct control over shared contracts, files, and transfer behavior | More initial implementation and test work; existing functionality must not be claimed before it is rebuilt and checked | Chosen, using established libraries where they meet the requirement |

[Jon Skeet's V-Drum Explorer](../current/acknowledgements.md) is a major reference for models, MIDI interfaces, schemas, firmware differences, tests, and failure cases. Its WPF GUI is platform-specific. VDrumExplorer-Cross-Platform already provides an Avalonia port, so its reported copy/paste, history, and kit-copy workflows must be considered. PulsoKit supplies another workflow and transfer reference, with a different platform scope. MIDI SYSEX-IO supplies generic GUI/CLI transfer tooling but does not replace a semantic kit editor. VDrumLib is historical prior art; its current scope and maintenance were not established.

The [acknowledgements and incorporation register](../current/acknowledgements.md) owns exact project revisions, researched findings, and any actual incorporations. Inspiration, independent interface implementation, translated/copied code, copied schema/data, and ordinary package dependencies must be identified separately. A translation is still an incorporation. No author code from those researched editors is currently incorporated.

## Consequences

Use established dependencies where they meet the requirement and independently implement the documented interface. Reconsider an individual incorporation when its concrete saving exceeds adaptation, licensing and maintenance cost. Do not start ongoing fork synchronization without demonstrated benefit. Preserve provenance and license obligations for every actual incorporation, including translations. The [current incorporation register](../current/acknowledgements.md) and [source register](../current/sources.md) record the material used; no compatibility or legal clearance follows merely from an acknowledgement.

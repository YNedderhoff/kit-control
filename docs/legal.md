# Legal and provenance record

## Current decision

Implement original hardware support through the documented TD-27 MIDI interface. Use original synthetic parameter values for emulation and public tests. Distinguish independently written interface code from redistribution of manufacturer material. An unresolved publication question does not impose a blanket ban on independent interface implementation.

Do not redistribute Roland manuals, graphics, firmware, samples, purchased kits, or personal backups. Apache 2.0 covers this project's original contributions; it grants no rights to third-party material or trademarks. Private use is not a blanket answer to legal questions.

## Sources and open questions

The [TD-27 MIDI implementation](https://static.roland.com/assets/media/pdf/TD-27_MIDI_Imple_eng04_W.pdf) identifies version 2.00, dated 11 August 2022. It is an interface reference, not a bundled project asset or proof of a device's installed firmware.

The [Roland Germany terms](https://www.roland.com/de/terms_of_use/) describe software-download licensing and related documentation. Their application to this implementation and any proposed redistribution requires a concrete review of the material and use. [UrhG §69a](https://www.gesetze-im-internet.de/urhg/__69a.html) distinguishes expression from underlying ideas and interface principles; it does not decide every contractual, copyright, or distribution question here. No general legal clearance is claimed.

| Question | Status and required decision |
| --- | --- |
| Documentation or catalog redistribution | Not permitted by project policy; link to the original source instead |
| Publication of independently written interface code | Assess any concrete rights concern against the exact material and use; do not conflate code with bundled manuals or sound content |
| KitControl name | Working name accepted for development. Preliminary search found existing software uses, including Niagara kitControl and facility software. No uniqueness or trademark clearance claim; resolve before public product release |
| Hardware captures and user files | Keep outside public fixtures; review owner permission, redaction, and provenance before any publication |
| Borrowed code | Review exact revision and file license, document changes, and preserve required notices before incorporation |

Record a specific rights concern with its source, affected artifact, intended use, responsible decision, and distribution restriction. Obtain qualified advice where needed. Do not use unsupported legal conclusions to claim either universal permission or a universal implementation ban.

## Dependencies and attribution

No author code from the researched drum-editor projects is incorporated. Original protocol code uses the documented interface; public emulator values are invented. Package dependencies retain their own licenses. See [acknowledgements](../ACKNOWLEDGEMENTS.md), [source register](sources.md), and [third-party notices](../third-party/README.md). Dependency review does not establish rights to unrelated manufacturer material.

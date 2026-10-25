# Verification record

This record distinguishes implemented behavior, automated checks, native runtime checks, and physical hardware tests. Local checks ran on 26 October 2026, macOS 26.6.2 ARM64, Rust 1.99.1 and Node.js 26.12.1. Git history identifies the source revision; CI results must be checked separately for the exact pushed commit.

## Local evidence

| Check | Result |
| --- | --- |
| Independent TD-27 packet tests | 27 passed against a test-only module mock with independent memory, framing and expectations |
| TD-27 CLI integration | 4 real subprocess tests passed against the production packet emulator |
| Offline core and CLI integration | 21 core tests and 12 subprocess tests passed |
| Complete Rust workspace | All 64 tests passed; formatting and Clippy with warnings denied passed |
| Frontend | 18 behavior tests passed; TypeScript and production bundle passed |
| macOS native package | Unsigned ARM64 app built at the final checkout location; current license texts and all five bundled MPL source archives checked against the source files and Cargo.lock |
| Native MIDI discovery | Standalone CLI and native desktop IPC both returned zero inputs and outputs with no device attached. CoreMIDI initialization requires execution outside the restricted agent shell sandbox |
| Native TD-27 GUI/client integration | Built app read the packet emulator, changed tempo 120 to 126 and enabled the snare compressor, displayed addresses `04 00 00 2E` and `04 01 41 01`, applied, and returned 2 sent / 2 verified / 2 total |
| Native offline workspace | Earlier app run changed snare tuning 0 to 7, reviewed/applied against the document simulator, and saved a five-file workspace; the resulting kit file contained tuning 7 |
| Linux / Windows | GitHub Actions configured; results are reported separately for the pushed commit |
| Physical Roland or other module | No physical transfer or recovery test; firmware of the personal TD-27 remains unknown |
| Save recovery | Process-interruption paths covered; power loss, full SD backup restoration, and sample recovery not tested |

## Independent TD-27 expectations

`crates/core/tests/td27.rs` contains a separate test-only memory-level module mock. It does not call production encoders, decoders, address helpers, or the production emulator to create expected replies. Fixed packet vectors are checked against the [TD-27 MIDI implementation v2.00](https://static.roland.com/assets/media/pdf/TD-27_MIDI_Imple_eng04_W.pdf); parameter values are invented.

Coverage includes directed identity and model checks; exact RQ1/DT1 bytes, checksums and ranges; kit-address carry through slot 100; the declared read scope; split, out-of-order, overlapping and missing responses; byte fragments and interleaved realtime bytes; stale and forged plans; changed native endpoints; documented name bytes; selective writes and preservation of unrelated data; disconnect after a completed write; readback failure; and final-state verification counts. The CLI tests exercise read/plan/apply, JSON errors and exit codes, filename collisions, and packet-emulator faults through the public executable.

The production emulator is a useful application test mode, not the sole correctness oracle. Passing client-to-emulator roundtrips alone would not establish protocol correctness.

## Review and limits

An independent review prompted fixes for native endpoint binding, documented nonprintable name bytes, stopping sends after a latched native queue overflow, and final verification counts. Regression tests cover endpoint binding, name preservation, and final read failure/mismatch. Overflow handling was reviewed in source; a physical native overflow was not induced. Re-review found no remaining actionable issue in those fixes. Earlier storage review prompted atomic no-clobber creation, mismatch-injection and journal-ordering fixes. The first macOS CI run exposed a numeric input lifecycle race in the offline editor. Numeric fields now reset by committed-value keys, without a competing passive effect. A delayed-core regression covers the first edit, pending state, undo/redo and load; the complete frontend suite passed twice after the fix.

The snapshot covers only Kit Common and twelve Pad blocks. It is neither a complete kit nor a sample backup. No physical module is attached. Native MIDI packet delivery, module persistence, real-world timing, and restoration remain unverified until the [hardware procedure](hardware-test-protocol.md) runs. Port names and baseline equality do not provide a unique physical serial identity or atomic device transactions. Screenshots and mocked UI tests do not prove hardware compatibility; a file checksum does not prove device restoration.

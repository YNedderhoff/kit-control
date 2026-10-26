# Private read-only hardware report template

Copy this blank template outside the app checkout before filling it in. Do not commit personal kit names, purchase files, paths, samples, captures or populated reports to the public app repository. Follow the [hardware procedure](hardware-test-protocol.md); this template does not authorize any device operation or upload.

Use `passed`, `failed` or `untested` for each check. Leave unknown facts explicitly unknown. Record expected values **before** the first MIDI request; never fill them from the actual snapshot. Record observer, time and evidence references privately.

## Session and prerequisites

| Field | Observation / evidence |
| --- | --- |
| Date, observer, session start/end, 90-minute deadline | Unknown |
| Owner authorization and approved read scope | Pending |
| App source commit; clean/modified tree; CLI path and SHA-256 | Unknown |
| Protocol/CLI test commands, source commit, results | Untested |
| Model; full displayed firmware/build; observation source/time | Unknown |
| Matching MIDI document URL, revision/date and page references | Unknown |
| OS/version/architecture; MIDI backend | Unknown |
| USB COMPUTER connection, cable/adapters | Unknown |
| Existing GENERIC/VENDOR mode; installed/active driver; OS support source/date | Unknown |
| Displayed MIDI device ID; API/wire value | Unknown |
| MIDI Tx/Rx, Soft Thru and sync settings | Unknown |
| Exact unique input/output names; other MIDI applications/routing excluded | Unknown |
| Current kit number, retained throughout capture | Unknown |

## Backup gate

| Check | Status | Private evidence / limits |
| --- | --- | --- |
| Existing SD and purchase/sample originals preserved before new SD writes | Untested | |
| Original archive inventory: relative paths, byte sizes, SHA-256 | Untested | |
| Current native BACKUP ALL: free bank, With User Sample selected, completion | Untested | |
| Current sample count, coverage/age and any uncertainty | Untested | |
| New backup archived and checked against inventory | Untested | |
| Local archive outside app checkout and retained SD originals | Untested | |
| Private remote visibility, file list/LFS limits reviewed; upload authorized | Untested | |
| Fresh remote clone and LFS fetch with isolated storage, no shared cache | Untested | |
| All inventoried files materialized at recorded revisions; sizes/hashes match | Untested | |
| Overall gate | Untested | No physical MIDI until passed |
| Device/module/sample restore | Untested | Not part of this session |

Record private archive/repository locations and data commit separately. State which actual binaries are held in the local archive and LFS storage, which text/references are in Git, and which tools/driver/settings must be recreated. File retrieval is not device recovery.

## Frozen independent expectations

For each parameter record the module menu/page, observer/time and expected value before MIDI access. For inaccessible values record the reason and keep comparison untested. Do not change settings to make a comparison possible.

| Parameter | Display/menu evidence | Expected | Observed read A / B | Status |
| --- | --- | --- | --- | --- |
| kit.name | | Unknown | | Untested |
| kit.tempo (stored kit tempo, not global click) | | Unknown | | Untested |
| kit.tempo_enabled | | Unknown | | Untested |
| kit.brush_enabled | | Unknown | | Untested |
| kit.xstick_enabled | | Unknown | | Untested |
| pad.kick.compressor_enabled | | Unknown | | Untested |
| pad.snare.compressor_enabled | | Unknown | | Untested |
| pad.tom1.compressor_enabled | | Unknown | | Untested |
| pad.tom2.compressor_enabled | | Unknown | | Untested |
| pad.tom3.compressor_enabled | | Unknown | | Untested |
| pad.hihat.compressor_enabled | | Unknown | | Untested |
| pad.crash1.compressor_enabled | | Unknown | | Untested |
| pad.crash2.compressor_enabled | | Unknown | | Untested |
| pad.ride.compressor_enabled | | Unknown | | Untested |
| pad.aux1.compressor_enabled | | Unknown | | Untested |
| pad.aux2.compressor_enabled | | Unknown | | Untested |
| pad.aux3.compressor_enabled | | Unknown | | Untested |

Separately freeze a documentation-derived list of the selected kit's **13 concrete addresses and sizes** before capture. Expected scope is 52 bytes plus twelve times 11 bytes, total 184. Record identity/model/device fields, name encoding, tempo nibble and switch encodings, checksum rules and exact document pages. Do not use production helpers to prepare this list. Unknown raw bytes have no semantic expectation.

## Execution and result

| Step | Exact command / exit code / stdout and stderr evidence | Status |
| --- | --- | --- |
| ports | | Untested |
| directed identify | | Untested |
| read A, new private destination | | Untested |
| read B, separate new private destination | | Untested |
| Native identity/endpoints/kit/simulation marker match | | Untested |
| Both captures contain expected addresses and all 184 bytes | | Untested |
| Repeated raw blocks and decoded values match | | Untested |
| Frozen display expectations match; post-read display unchanged | | Untested |

Record stop reason/time, partial completion, missing evidence and unexecuted steps. Do not infer no hardware contact from a failed command or missing snapshot file. Record packet counts/types as observed only if a capture exists; otherwise identify them as source-inspected expectations. Scope allows directed Identity Request/RQ1 outbound and DT1 replies inbound, with no outbound DT1 or control messages.

State the exact parameter subset compared, inaccessible display fields, stable unknown bytes and any discrepancies. Device writes, copies/reorders, persistence and full module/sample recovery remain untested. Do not retry, repair or restore automatically after failure.

## Evidence separation and disclosure

| Evidence category | Source revision / result / limits |
| --- | --- |
| Implemented profile and CLI | |
| Independent mock protocol tests | |
| CLI emulator integration tests | |
| Virtual native CoreMIDI integration, if separately run | |
| Physical TD-27 read and display comparison | |
| File retrieval and hash verification | |
| Device restore | Untested |
| Exact-commit CI (passed/failed/skipped/running/unreadable) | |

Overall outcome: **untested**. A successful read verifies only the captured scope on this model/firmware/host/connection/commit. A partial display comparison must name its verified subset.

Disclosure review: **not approved**. Keep this populated report private. Prepare a separate redacted summary and obtain approval before publishing it or changing public support claims.

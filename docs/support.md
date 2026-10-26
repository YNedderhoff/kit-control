# Support and verification

| Area | Intended scope | Verified support |
| --- | --- | --- |
| Offline document operations | Original synthetic data only | Local Rust integration tests; details in verification record |
| Synthetic document simulator | Offline workspace development mode | Local core/CLI integration and native macOS workflow |
| TD-27 packet emulator | Same MIDI client as native transport; invented parameter values and fault injection | Independent packet, CLI, UI and native macOS emulator checks; see verification record |
| macOS desktop | Tauri 2 and React | ARM64 app built and native edit/plan/apply/save workflow checked |
| Linux desktop | Tauri 2 and React | CI build/package passed for the recorded revision; native GUI/MIDI runtime not yet verified |
| Windows desktop | Tauri 2 and React | CI build/package passed for the recorded revision; native GUI/MIDI runtime not yet verified |
| Roland TD-27 firmware 2.00 | Native MIDI identity, partial snapshot, planned supported-field writes, readback | No physical device test recorded |
| Other firmware / modules | Outside the initial profile | Unsupported |
| Roland backup formats and samples | Outside the implemented parameter scope | None |
| Mobile | Later work | None |

The TD-27 snapshot includes kit name, tempo and switches plus twelve pad compressor switches. It is not a complete kit or sample backup. Native tuning, kit copy, and reordering remain pending. The user must confirm exact firmware `2.00` from the module; MIDI identity revision alone is insufficient.

See [verification.md](verification.md) for actual scope and [api.md](api.md) for simulator limitations. Platform builds do not establish real MIDI or hardware support.

When reporting a problem, include the commit, operating system, exact command or UI steps, expected result, actual result, and a minimal synthetic example. Remove personal information and secrets. Do not attach proprietary backups, samples, or captured device data without prior permission and review.

Maintainers review changes manually. There is no promised response time, hosted service, telemetry requirement, or release support commitment. Do not create remote issues or send reports on someone else's behalf without their authorization.

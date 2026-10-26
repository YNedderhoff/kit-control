# Contributing

Keep changes small and trace the affected flow before editing. Follow the existing Rust and frontend conventions. The shared Rust dispatcher owns business behavior; CLI and desktop callers must use it.

Implement hardware support through documented MIDI interfaces. Record exact interface sources and independently check protocol expectations. Use invented parameter values for emulator fixtures and public tests. Do not redistribute Roland manuals, graphics, firmware, samples, purchased kits, personal backups, secrets, or private infrastructure information. Record concrete rights and publication questions in the [legal and provenance record](docs/current/legal.md); unresolved redistribution questions do not impose a blanket ban on original interface code. Review provenance and license obligations before incorporating third-party material.

Tests must specify expected values independently of production constants and logic. Cover relevant invalid input, stale state, partial completion, and readback failure. Test through the affected public integration, and report exactly which checks ran. Simulator success does not establish hardware compatibility.

Before completion, review the final diff for correctness, maintainability, security, and affected documentation. For non-trivial changes, obtain an independent review when available. Maintainers remain responsible for manual review of AI-assisted changes. Changes must pass the applicable CI checks and receive human approval before merging; do not enable automatic merges. The [dependency operating guide](docs/current/dependencies.md) describes the GitHub check gate and its separate activation verification.

Use Conventional Commits with English imperative titles of at most 72 characters. Add an `AI-Model:` trailer for AI-assisted commits. Use an exact identifier only when known reliably; otherwise record the known family with `(exact variant unavailable)`, or `unknown (tool name)` if the family is unknown. Never infer a model from an available-model list.

Run relevant checks before committing or pushing. After an authorized push, inspect CI for that exact commit and distinguish passed, failed, skipped, running, and unreadable checks. Local checks do not establish CI success. Ask the owner before creating remote repositories, pull requests, issues, messages, releases, deployments, or deleting data.

Submitted contributions are under Apache License 2.0 unless explicitly agreed otherwise. Do not submit material you lack permission to contribute.

## Documentation lifecycle

Use the [documentation index](docs/README.md) before each change. This structure is required for future commits, not merely a one-time file layout.

| Content | Location | Purpose and status |
| --- | --- | --- |
| Open architecture proposals | `docs/rfcs/YYYY-MM-DD-topic.md` | Requirements, proposal, alternatives and open questions; draft, ready for decision, accepted, rejected, withdrawn or superseded |
| Accepted architecture decisions | `docs/adrs/YYYY-MM-DD-topic.md` | Context, decision, alternatives and consequences; accepted, superseded or revoked |
| Concrete implementation work | `docs/projects/YYYY-MM-DD-topic.md` | Scope, dependencies, steps and acceptance; planned, in progress, completed or cancelled |
| Existing implementation, use and limits | `docs/current/` | Undated names; maintain actual behavior and verification boundaries |

Root README, CONTRIBUTING and AGENTS files remain discovery/contribution entry points. Root license/generated notices and `third-party/` source-availability material remain distribution assets. Substantive project documentation belongs in the four directories above; `docs/README.md` is their index. Rust source stays under `crates/`, React/Tauri under `apps/desktop/`, test-only fixtures with the relevant tests, and CI/templates under `.github/`. Add crates only when responsibilities warrant them.

The owner decides architecture and the scope of RFCs/projects. Propose unclear classifications rather than creating speculative work or a document for every step. An ADR can exist without an earlier RFC. Do not invent retrospective RFCs for already accepted choices.

An RFC remains under its original name when decided and links to the resulting ADR; it is not renamed into an ADR. Partial acceptance must distinguish accepted scope from open questions. An ADR must explain its decision independently. After acceptance, preserve that decision; a changed direction gets a linked replacement ADR. Corrections and references may be updated without rewriting the original choice. Accepted does not mean implemented, installed or physically tested.

File dates record RFC creation, the decision, or project start. Do not change them on routine edits. Use the actual owner decision date where evidence exists. If a historical date truly cannot be established, mark that gap explicitly rather than inventing one. Current operational/software pages use undated names and state actual limits.

Before each commit:

1. Read the relevant RFC/ADR/project indexes and find affected decisions, work, callers, sources and current pages. Existing plans do not authorize extra implementation or external actions.
2. Update the existing project scope, dependencies, remaining steps and acceptance evidence for the work actually done. A partial step must not close the whole project. Change an RFC's status only for an actual decision. Update affected category indexes when adding, replacing or closing records.
3. Update affected current behavior, usage, source/provenance, backup/recovery limits and cross-references in the same commit. Record what comes from Git, what needs separate backup, and what must be recreated. Preserve paid kits and original sample backups in the separate private data repository.
4. Keep one authoritative place per fact: proposal detail in the RFC, the decision in the ADR, work/status in the project, and existing behavior in current documentation. Short context and links are useful; duplicated inventories or temporary handoff snapshots are not. Closed projects retain concise acceptance and links rather than obsolete install instructions.
5. Check moved Markdown links, anchors, plain file paths, source comments, packaging and notice references. Keep all accepted requirements and required attribution. Run relevant checks, inspect the final diff, then follow the owner's commit/push authorization and inspect the exact commit's CI.

Human responsibility, independent expected values, transparent AI attribution and meaningful automated tests remain required. Welcome test reports, issues and PRs for additional modules without overstating support or publishing on someone's behalf. Use the supplied general contribution rules; do not copy private infrastructure material or invent unread policies.

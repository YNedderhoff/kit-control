# Contributing

Keep changes small and trace the affected flow before editing. Follow the existing Rust and frontend conventions. The shared Rust dispatcher owns business behavior; CLI and desktop callers must use it.

Implement hardware support through documented MIDI interfaces. Record exact interface sources and independently check protocol expectations. Use invented parameter values for emulator fixtures and public tests. Do not redistribute Roland manuals, graphics, firmware, samples, purchased kits, personal backups, secrets, or private infrastructure information. Record concrete rights and publication questions in the [legal and provenance record](docs/legal.md); unresolved redistribution questions do not impose a blanket ban on original interface code. Review provenance and license obligations before incorporating third-party material.

Tests must specify expected values independently of production constants and logic. Cover relevant invalid input, stale state, partial completion, and readback failure. Test through the affected public integration, and report exactly which checks ran. Simulator success does not establish hardware compatibility.

Before completion, review the final diff for correctness, maintainability, security, and affected documentation. For non-trivial changes, obtain an independent review when available. Maintainers remain responsible for manual review of AI-assisted changes. Changes must pass the applicable CI checks and receive human approval before merging; do not enable automatic merges. These are contribution requirements, not a claim that repository branch protection has been configured.

Use Conventional Commits with English imperative titles of at most 72 characters. Add an `AI-Model:` trailer for AI-assisted commits. Use an exact identifier only when known reliably; otherwise record the known family with `(exact variant unavailable)`, or `unknown (tool name)` if the family is unknown. Never infer a model from an available-model list.

Run relevant checks before committing or pushing. After an authorized push, inspect CI for that exact commit and distinguish passed, failed, skipped, running, and unreadable checks. Local checks do not establish CI success. Ask the owner before creating remote repositories, pull requests, issues, messages, releases, deployments, or deleting data.

Submitted contributions are under Apache License 2.0 unless explicitly agreed otherwise. Do not submit material you lack permission to contribute.

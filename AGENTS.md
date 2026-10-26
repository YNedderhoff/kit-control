# KitControl agent rules

- Use plain, concise language. State assumptions, actual checks, and unknowns.
- Trace affected callers, documents, contracts, and existing decisions before editing. Keep changes focused and update affected docs in the same change.
- Read `docs/plan.md` for the accepted scope, `docs/roadmap.md` for the audited implementation state, and `docs/handoff.md` before continuing a new task. Plans record requirements; they do not authorize unrequested execution or external publication.
- The Rust core owns editing and plan/apply/verify behavior. CLI and Tauri use the same JSON dispatcher. Do not duplicate business rules in React or CLI presentation code.
- Implement real hardware support through documented MIDI interfaces. Use original synthetic parameter values for emulation and public tests. Record interface sources and independently check protocol expectations. Do not redistribute Roland manuals, graphics, firmware, samples, purchased kits or personal backups. Track concrete rights questions in `docs/legal.md`; do not turn an unresolved publication question into a blanket ban on independent interface implementation.
- Offline edits cannot write to a device. Preserve explicit plan review, apply, and readback verification, including stale-plan and failure behavior.
- Never claim hardware, desktop, platform, recovery, or CI verification from a different check. Record tested commit and actual scope.
- Keep test fixtures separate from production data. Specify expected values independently of production constants and logic. Cover relevant invalid and failure paths through public integrations.
- Review the final diff. Seek independent review for non-trivial work when available; maintainers remain responsible for manual review.
- Do not publish user data, private infrastructure, or secrets. Do not create remote repositories, PRs, issues, messages, releases, deployments, or delete data without owner approval.
- Run relevant checks before committing or pushing. Use Conventional Commits, titles at most 72 characters, and truthful `AI-Model:` trailers. Do not force-push. After an authorized push, inspect CI for that exact commit and report its actual status.
- Honor the owner's task-specific commit and push instructions. Do not treat this file as authorization to publish.

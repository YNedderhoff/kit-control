# Separate proposals, decisions, implementation work and current documentation

- Status: accepted.
- Decision date: 2026-10-26.
- Recorded: 2026-10-26.
- Decision basis: owner approval to place all project documentation into the RFC/ADR/project/current structure and make that structure binding for future commits.

## Context

A single development plan mixed decisions, future work, current implementation and verification. A task-specific handoff name also made a permanent document look like a temporary snapshot. New contributors need a durable project record without inherited task context.

## Decision

Use `docs/rfcs/` for open proposals, `docs/adrs/` for accepted decisions and their rationale, `docs/projects/` for execution scope/dependencies/acceptance, and `docs/current/` for existing implementation, use, evidence and limits. [docs/README.md](../README.md) is the entry point. Keep conventional repository entry points and legally distributed notices at their required locations.

The [contribution guide](../../CONTRIBUTING.md#documentation-lifecycle) owns placement, statuses, maintenance and date rules. Agent instructions point there. Before a change, identify relevant RFCs, ADRs, projects and current pages; update affected work, sources, links and factual statements in the same commit. A decision being accepted does not mean implementation or hardware acceptance is complete.

## Alternatives

A flat collection of plans and handoffs leaves ownership unclear and encourages snapshots or duplicated status. Renaming a handoff alone would not separate decisions from changing implementation work. Creating an RFC for every routine step would add work without a decision to resolve.

## Consequences

Keep existing approved requirements and evidence, with one authoritative location per fact. Record decisions dated from the owner's instructions rather than inventing dates from the later documentation migration. Preserve accepted ADRs; a changed direction receives a linked replacement. Create new RFCs/projects only when their purpose and scope warrant them. This convention is adapted for an application repository; it imports no private infrastructure configuration or operating rules.

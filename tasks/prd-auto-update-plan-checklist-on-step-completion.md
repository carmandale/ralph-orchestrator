# PRD: Auto-Update Plan Checklist on Step Completion

## Overview
Automatically update the plan checklist when steps complete so the plan stays in sync with actual progress, preventing reruns. Updates should occur on step completion events and reconcile on LOOP_COMPLETE. The plan path must come from session metadata/config (no glob search or PROMPT.md path discovery).

## Goals
- Keep `plan.md` checklists accurate after each loop.
- Prevent re-running already completed steps.
- Handle multi-step sessions by marking all completed steps.
- Use session-stored plan paths for any plan location.

## Quality Gates

These commands must pass for every user story:
- `cargo build`
- `cargo test`

## User Stories

### US-001: Update plan checklist on step completion events
**Description:** As the orchestrator, I want completed steps marked in the plan checklist when step completion events occur so the plan reflects real-time progress.

**Acceptance Criteria:**
- [ ] Use the plan path stored in session metadata/config to locate the active plan file.
- [ ] When a step completion event is emitted, update the matching checklist item from `- [ ]` to `- [x]`.
- [ ] Only Markdown task list items (`- [ ]` / `- [x]`) are eligible for updates.
- [ ] If no matching checklist item is found, log a warning and continue without failing the loop.
- [ ] Reprocessing the same completion event does not change already-checked items (idempotent).

### US-002: Reconcile plan checklist on LOOP_COMPLETE
**Description:** As the orchestrator, I want to reconcile the plan checklist after LOOP_COMPLETE so multi-step sessions are fully reflected in the plan.

**Acceptance Criteria:**
- [ ] On LOOP_COMPLETE, parse the session’s completion summary (from PROMPT.md or the loop’s recorded summary) to determine all steps completed in the loop.
- [ ] All completed steps identified in the summary are marked `- [x]` in the plan.
- [ ] If the completion summary does not contain a recognizable step list, log a warning and leave the plan unchanged.
- [ ] Works when multiple steps are completed in a single session (e.g., steps 1-4).

### US-003: Derive next step from updated plan checklist
**Description:** As the orchestrator, I want to compute the next step from the plan checklist so the correct step is selected for the next loop.

**Acceptance Criteria:**
- [ ] After checklist updates, the next step is the first unchecked item (`- [ ]`) in the plan.
- [ ] If there are no unchecked items, the plan is treated as complete for next-step selection.
- [ ] Next-step detection uses the same plan path from session metadata/config.

## Functional Requirements
- FR-1: The system must read the active plan path from session metadata/config, not via glob search or PROMPT.md path parsing.
- FR-2: On step completion events, the system must mark the corresponding checklist item as completed in the plan.
- FR-3: On LOOP_COMPLETE, the system must reconcile the plan against the session completion summary and mark all completed steps.
- FR-4: Only Markdown task list items (`- [ ]` / `- [x]`) are considered for updates.
- FR-5: If a completed step does not match a checklist item, the system must log a warning and continue.
- FR-6: Next-step detection must select the first unchecked item in the plan after updates.

## Non-Goals
- Supporting alternative checklist formats (e.g., `* [ ]`, `✅`, or custom markers).
- Reordering or rewriting the plan structure beyond toggling checklist state.
- Introducing compatibility layers for legacy or non-standard plan formats.
- Changing how plans are generated or stored.

## Technical Considerations
- The plan file is typically under a session path like `.../.ralph-o/sessions/<session-id>/plan/implementation/plan.md` and should be resolved from session metadata/config.
- File writes should preserve all non-checklist content and only toggle checkbox state on the matching line.
- Prefer step completion event payloads for matching when available; use LOOP_COMPLETE reconciliation as a fallback.

## Success Metrics
- No reruns of already completed steps due to stale plan checklists.
- After each loop, the plan checklist accurately reflects completed steps.
- Multi-step sessions consistently update all completed steps in one reconciliation.

## Open Questions
- What is the exact format of the completion summary in PROMPT.md (or elsewhere) that should be parsed on LOOP_COMPLETE?
- If the plan path is missing or the file is unreadable, should this be a warning or a hard error?
- Is there any in-memory plan cache that also needs to be updated alongside the file write?
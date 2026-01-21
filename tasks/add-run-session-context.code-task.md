---
status: pending
created: 2026-01-21
started: null
completed: null
target_repo: null
---
# Task: Add Session Context Display to ralph-o run

## Description

The `ralph-o run` command should display session context at startup like `plan` and `task` commands do. Currently it only logs session info via `info!()` which requires `--verbose` flag.

## Background

For consistency across the ralph-o workflow:
- `ralph-o plan` shows: "Starting PDD session: {id}" + session dir
- `ralph-o task` shows: "Starting Code Task Generator session: {id}" + Feature + Plan path
- `ralph-o run` shows: nothing (only verbose logging)

Users need to see what session/prompt they're about to execute without hunting for it.

## Technical Requirements

1. Add startup output to `run_command()` in `crates/ralph-cli/src/main.rs`
2. Display session ID, feature description, PROMPT path, and tasks directory
3. Use same color formatting pattern as `plan` and `task` commands
4. Support both colored and non-colored output modes

## Implementation Approach

1. Locate session setup in `run_command()` (around line 624)
2. Add println statements after session is resolved, before loop starts
3. Follow the pattern from `task_command()` (lines 1334-1358)

## Expected Output

```
🚀 Starting execution: 001-planning-session
   Feature: Enhance Termination Output...
   PROMPT: .ralph-o/sessions/001-planning-session/PROMPT.md
   Tasks: .ralph-o/sessions/001-planning-session/tasks/
```

## Acceptance Criteria

1. **Session context displayed**
   - Given a session-based run
   - When `ralph-o run` starts
   - Then session ID, feature, PROMPT path, and tasks dir are shown

2. **Color support**
   - Given `--color=always` or `--color=never`
   - When `ralph-o run` starts
   - Then colors are applied/omitted correctly

3. **Consistency with other commands**
   - Given the output format
   - When compared to `plan` and `task` commands
   - Then the style and formatting are consistent

## Metadata
- **Complexity**: Low
- **Labels**: UX, CLI, consistency
- **Required Skills**: Rust, string formatting

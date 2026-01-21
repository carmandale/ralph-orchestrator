---
status: completed
created: 2026-01-21
started: 2026-01-21
completed: 2026-01-21
target_repo: null
---
# Task: Fix Legacy .agent/ Directory Writes

## Description

Something is still writing files to the legacy `.agent/` directory instead of the session directory (`.ralph-o/sessions/<id>/`). This causes the migration logic to trigger repeatedly and can corrupt session state.

## Background

After migrating from `.agent/` to session-based organization, files are still being written to `.agent/`:
- `summary.md` was found in `.agent/` after a run
- `events.jsonl` was found in `.agent/` after a run

The session directory (`.ralph-o/sessions/001-xxx/`) should be the only location for these files.

## Technical Requirements

1. Find all code paths that write to `.agent/` directory
2. Ensure all writes go to the session directory instead
3. Remove any hardcoded `.agent/` paths
4. Verify `SummaryWriter` uses the session's summary path
5. Verify event logging uses the session's events path

## Investigation Steps

1. Search for `.agent` string in codebase
2. Check `SummaryWriter::write()` - where does it write?
3. Check event logging - where does it write `events.jsonl`?
4. Check config defaults - does `RalphConfig` default to `.agent/`?
5. Trace the flow from `run_command()` to file writes

## Likely Culprits

- `SummaryWriter` may have hardcoded default path
- `RalphConfig::default()` may set `.agent/` paths
- Event logger may not be using session paths

## Acceptance Criteria

1. **No .agent/ writes**
   - Given a fresh run with `ralph-o run -p "test"`
   - When the run completes
   - Then no `.agent/` directory is created

2. **Session paths used**
   - Given a session-based run
   - When summary.md is written
   - Then it appears in `.ralph-o/sessions/<id>/summary.md`

3. **Events in session**
   - Given a session-based run
   - When events are logged
   - Then they appear in `.ralph-o/sessions/<id>/events.jsonl`

4. **Config paths respected**
   - Given session paths set in config
   - When SummaryWriter and event logger run
   - Then they use the config paths, not hardcoded defaults

## Metadata
- **Complexity**: Medium
- **Labels**: Bug, Session Management, Legacy Cleanup
- **Required Skills**: Rust, file I/O, config management

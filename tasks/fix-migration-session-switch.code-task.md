---
status: completed
created: 2026-01-21
started: 2026-01-21
completed: 2026-01-21
target_repo: ralph-orchestrator
---
# Task: Fix Migration Logic Session Switch Bug

## Description

The legacy `.agent/` migration logic incorrectly switches the current session pointer even when a current session already exists. This causes users to lose their working context after migration runs.

## Background

When `ralph-o run` detects a legacy `.agent/` directory:
1. It creates a new session `000-migrated` with the old contents
2. It sets `000-migrated` as the current session
3. This overwrites the existing current session pointer

**Observed behavior:**
```
$ ralph-o run
⚠️  Legacy .agent/ directory detected!
   Migrating to new session-based structure...
   ✓ Migration complete! Contents moved to: 000-migrated
   ✓ Old .agent/ directory removed

Error: Prompt file '.ralph-o/sessions/000-migrated/PROMPT.md' not found.
```

The user was working on `001-planning-session` but migration switched them to `000-migrated`.

## Technical Requirements

1. Migration should NOT change current session if one already exists
2. Migration should only set current to migrated session if no current session exists
3. Migration should warn user about the migrated session location without switching to it

## Implementation Approach

1. Locate migration logic (likely in `run_command()` or `SessionManager`)
2. Before calling `set_current()` for migrated session, check if current session exists
3. If current exists, skip `set_current()` and just inform user
4. If no current exists, set migrated session as current (existing behavior)

## Expected Behavior After Fix

```
$ ralph-o run
⚠️  Legacy .agent/ directory detected!
   Migrating to new session-based structure...
   ✓ Migration complete! Contents moved to: 000-migrated
   ✓ Old .agent/ directory removed
   ℹ️  Keeping current session: 001-planning-session
       (Migrated content available in 000-migrated)

🚀 Starting execution: 001-planning-session
   ...
```

## Acceptance Criteria

1. **Current session preserved**
   - Given an existing current session (e.g., 001-planning-session)
   - When migration runs on legacy .agent/ directory
   - Then current session pointer remains unchanged

2. **Migration still works**
   - Given a legacy .agent/ directory
   - When migration runs
   - Then contents are moved to 000-migrated session

3. **User informed of migration**
   - Given migration with existing current session
   - When migration completes
   - Then user sees message about migrated content location

4. **No current session fallback**
   - Given no current session exists
   - When migration runs
   - Then migrated session becomes current (existing behavior)

## Metadata
- **Complexity**: Low
- **Labels**: Bug, Migration, Session Management
- **Required Skills**: Rust, session management

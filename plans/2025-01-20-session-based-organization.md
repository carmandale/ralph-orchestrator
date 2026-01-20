# Session-Based Organization for Ralph-O

## Overview

Reorganize ralph-o to use session-based directory structure under `.ralph-o/sessions/`. Each session is a self-contained unit with its own PROMPT.md, scratchpad.md, events.jsonl, and planning artifacts. This enables clean history, no cross-contamination between sessions, and easy reference to past work.

## Problem Statement

Currently ralph-o stores all artifacts in a flat `.agent/` directory:
- `PROMPT.md` lives in project root (gets overwritten)
- `.agent/scratchpad.md` is shared across all runs
- `.agent/events.jsonl` accumulates indefinitely
- No history of past sessions or their prompts
- Planning artifacts from `ralph-o plan` go to `.sop/planning/` (separate location)

This creates problems:
1. **No session history** — Can't reference what you worked on before
2. **Cross-contamination** — Scratchpad from one task bleeds into another
3. **Lost prompts** — Previous PROMPT.md files get overwritten
4. **Scattered artifacts** — Planning output separate from execution output

## Research Findings

### Existing Patterns

- `crates/ralph-core/src/config.rs:580` — Default scratchpad: `".agent/scratchpad.md"`
- `crates/ralph-core/src/event_logger.rs:139` — Default events: `".agent/events.jsonl"`
- `crates/ralph-core/src/summary_writer.rs:45` — Default summary: `".agent/summary.md"`
- `crates/ralph-core/src/workspace.rs:117` — Benchmark workspaces create `.agent/` per workspace
- `.claude/skills/pdd/SKILL.md` — PDD skill outputs to `.sop/planning/`

### Key Config Locations

```rust
// crates/ralph-core/src/config.rs
fn default_scratchpad() -> String {
    ".agent/scratchpad.md".to_string()
}

// crates/ralph-core/src/event_logger.rs
pub const DEFAULT_PATH: &'static str = ".agent/events.jsonl";

// crates/ralph-core/src/summary_writer.rs
Self::new(".agent/summary.md")
```

### Dependencies

- `ralph-core` — Config, EventLogger, SummaryWriter, EventReader
- `ralph-cli` — init, run, resume, clean, events commands
- PDD skill — `.claude/skills/pdd/SKILL.md`

### Constraints

- Must maintain backwards compatibility (or clear migration path)
- Session IDs need to be deterministic and human-readable
- Need to handle `ralph-o resume` knowing which session to resume

## Proposed Solution

### New Directory Structure

```
.ralph-o/
├── config.yml                    # Project config (replaces ralph.yml in root)
└── sessions/
    ├── 001-rest-api/
    │   ├── PROMPT.md             # The task prompt
    │   ├── scratchpad.md         # Session-specific working memory
    │   ├── events.jsonl          # Session event log
    │   ├── summary.md            # Completion summary
    │   └── plan/                 # Planning artifacts (from ralph-o plan)
    │       ├── idea-honing.md
    │       ├── design/
    │       └── implementation/
    │
    ├── 002-auth-feature/
    │   └── ...
    │
    └── current -> 002-auth-feature/   # Symlink to active session
```

### Session Naming

Format: `NNN-<kebab-title>`
- `NNN` — Zero-padded sequential number (001, 002, ...)
- `<kebab-title>` — Derived from prompt or user-provided

Examples:
- `001-rest-api`
- `002-add-authentication`
- `003-fix-login-bug`

### Approach

#### Phase 1: Core Infrastructure

1. Add `Session` struct to manage session paths
2. Add `SessionManager` to create/list/get sessions
3. Update `RalphConfig` to support session-based paths
4. Add `current` symlink management

#### Phase 2: CLI Updates

1. `ralph-o init` — Creates `.ralph-o/config.yml` (no more `ralph.yml` in root)
2. `ralph-o plan "description"` — Creates new session with PROMPT.md and plan/
3. `ralph-o run` — Runs current session (errors if no session exists)
4. `ralph-o run --session 001` — Run specific session
5. `ralph-o resume` — Resumes current session
6. `ralph-o resume --session 002` — Resume specific session
7. `ralph-o list` — Show all sessions with status
8. `ralph-o clean --session 001` — Clean specific session (manual only)

#### Phase 3: Planning Integration

1. Update PDD skill to output to session's `plan/` directory
2. `ralph-o plan` creates a new session and populates `plan/`
3. PROMPT.md generated in session directory, not project root

### Files to Create/Modify

- [ ] `crates/ralph-core/src/session.rs` — New: Session and SessionManager
- [ ] `crates/ralph-core/src/config.rs` — Add session support
- [ ] `crates/ralph-core/src/event_logger.rs` — Use session path
- [ ] `crates/ralph-core/src/event_reader.rs` — Use session path
- [ ] `crates/ralph-core/src/summary_writer.rs` — Use session path
- [ ] `crates/ralph-cli/src/main.rs` — Add list command, --session flag
- [ ] `crates/ralph-cli/src/init.rs` — Update defaults
- [ ] `.claude/skills/pdd/SKILL.md` — Update output paths

### Implementation Steps

1. [ ] Create `Session` struct with path helpers
2. [ ] Create `SessionManager` for CRUD operations
3. [ ] Add session config to `RalphConfig`
4. [ ] Update `EventLogger` to accept session path
5. [ ] Update `EventReader` to accept session path
6. [ ] Update `SummaryWriter` to accept session path
7. [ ] Add `ralph-o list` command
8. [ ] Add `--session` flag to run/resume
9. [ ] Update `ralph-o clean` for session support
10. [ ] Update PDD skill for session integration
11. [ ] Add migration logic for existing `.agent/` directories
12. [ ] Update tests
13. [ ] Update documentation

## Technical Considerations

### Architecture Impact

This is a moderate refactor touching core path handling. The `Session` abstraction provides a clean boundary — all path lookups go through the session rather than hardcoded strings.

### Backwards Compatibility

Options:
1. **Migration on first run** — Detect `.agent/`, offer to migrate to `.ralph-o/sessions/000-migrated/`
2. **Parallel support** — Support both `.agent/` (legacy) and `.ralph-o/` (new)
3. **Clean break** — v2.2 uses `.ralph-o/`, users migrate manually

Recommend: Option 1 (migration on first run) with clear messaging.

### Performance

Minimal impact. Session lookup is a directory scan, cached after first access.

## Acceptance Criteria

- [ ] `ralph-o init` creates `.ralph-o/config.yml` (not `ralph.yml` in root)
- [ ] `ralph-o plan "build X"` creates `.ralph-o/sessions/001-build-x/` with PROMPT.md and plan/
- [ ] `ralph-o run` without a session errors: "No session found. Run `ralph-o plan` first."
- [ ] `ralph-o run` with current session runs it, stores artifacts in session dir
- [ ] `ralph-o list` shows all sessions with number, title, status, date
- [ ] `ralph-o run --session 001` runs specific session
- [ ] `ralph-o resume` resumes current session
- [ ] `ralph-o resume --session 002` resumes specific session
- [ ] `ralph-o clean --session 001` cleans specific session (no auto-cleanup)
- [ ] Sessions are isolated — no shared scratchpad between sessions
- [ ] Session numbers are per-project (001, 002, ... per repo)
- [ ] Old `.agent/` and `ralph.yml` detected with migration offered

## Testing Strategy

- [ ] Unit tests for `Session` and `SessionManager`
- [ ] Integration tests for CLI commands with sessions
- [ ] Migration tests for `.agent/` → `.ralph-o/` conversion
- [ ] Manual testing: full workflow from plan → run → resume

## Success Metrics

- Clean session history visible via `ralph-o list`
- No more scratchpad cross-contamination
- Ability to reference and resume past sessions by number
- Planning and execution artifacts co-located

## Design Decisions

| Question | Decision |
|----------|----------|
| Session numbers | Per-project (001, 002, ... resets per repo) |
| Auto-cleanup | No — manual cleanup only |
| Config location | Move to `.ralph-o/config.yml` |
| `run` without `plan` | Error — must run `ralph-o plan` first |

## References

- Existing workspace isolation: `crates/ralph-core/src/workspace.rs`
- Pi agent plan skill: `~/.pi/agent/skills/plan/SKILL.md`
- PDD skill: `.claude/skills/pdd/SKILL.md`
- Related bead: `ralph-orchestrator-aju`

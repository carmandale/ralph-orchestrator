---
status: completed
created: 2026-01-20
started: 2026-01-20
completed: 2026-01-20
---
# Task: Implement derive_status and Module Export

## Description
Implement the `derive_status()` function that determines a session's status by inspecting its directory contents, and export the session module from `lib.rs`.

## Background
Session status can be derived from the presence and content of files in the session directory. This allows Ralph to reconstruct session state without requiring explicit status tracking. The derivation logic inspects artifacts like events.jsonl and summary.md to determine whether a session is in planning, in progress, completed, failed, or abandoned.

## Reference Documentation
**Required:**
- Design: .sop/planning/design/detailed-design.md

**Note:** You MUST read the detailed design document before beginning implementation. It specifies the status derivation rules.

## Technical Requirements
1. Implement `Session::derive_status(path: &Path) -> SessionStatus` function
2. Status derivation rules:
   - **Planning**: Only PROMPT.md exists, no events.jsonl
   - **InProgress**: events.jsonl exists but no summary.md
   - **Completed**: summary.md exists (indicates successful completion)
   - **Failed**: events.jsonl contains error events (check last event)
   - **Abandoned**: Directory exists but appears stale (optional heuristic)
3. Add `mod session;` to `crates/ralph-core/src/lib.rs`
4. Add public exports: `pub use session::{Session, SessionStatus};`

## Dependencies
- Tasks 1 and 2 (Session struct and path helpers must exist)
- File system access for directory inspection
- JSON parsing for events.jsonl inspection (serde_json)

## Implementation Approach
1. Implement `derive_status()` as a static method or associated function
2. Check for file existence in order of precedence
3. For Failed status, read the last line of events.jsonl and check for error indicators
4. Add the module declaration and exports to lib.rs
5. Add comprehensive unit tests using temp directories

## Acceptance Criteria

1. **Planning Status**
   - Given a session directory with only PROMPT.md
   - When calling derive_status()
   - Then it returns SessionStatus::Planning

2. **InProgress Status**
   - Given a session directory with PROMPT.md and events.jsonl (no summary.md)
   - When calling derive_status()
   - Then it returns SessionStatus::InProgress

3. **Completed Status**
   - Given a session directory with summary.md present
   - When calling derive_status()
   - Then it returns SessionStatus::Completed

4. **Failed Status**
   - Given a session directory where events.jsonl ends with an error event
   - When calling derive_status()
   - Then it returns SessionStatus::Failed

5. **Module Export**
   - Given the session module added to lib.rs
   - When importing from ralph_core
   - Then Session and SessionStatus are accessible as `ralph_core::Session` and `ralph_core::SessionStatus`

6. **Unit Test Coverage**
   - Given various directory states created in temp directories
   - When running `cargo test -p ralph-core session`
   - Then all status derivation tests pass

7. **Build Verification**
   - Given the complete session module
   - When running `cargo build -p ralph-core`
   - Then the build succeeds with no errors

## Metadata
- **Complexity**: Medium
- **Labels**: Core, Session, Status, Module Export
- **Required Skills**: Rust, file system operations, JSON parsing, module system

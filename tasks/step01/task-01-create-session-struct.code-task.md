---
status: completed
created: 2026-01-20
started: 2026-01-20
completed: 2026-01-20
---
# Task: Create Session Struct and SessionStatus Enum

## Description
Create the core `Session` struct that represents a single Ralph session and the `SessionStatus` enum that tracks session lifecycle states. This is the foundation for the session-based organization feature.

## Background
Ralph is moving from a single `.agent/` directory to a session-based organization where each task gets its own isolated directory under `.ralph-o/sessions/`. The `Session` struct is the core data type that represents one of these sessions, holding metadata like the session ID, title, path, and status.

## Reference Documentation
**Required:**
- Design: .sop/planning/design/detailed-design.md

**Note:** You MUST read the detailed design document before beginning implementation. It contains the complete specification for the Session struct fields and SessionStatus variants.

## Technical Requirements
1. Create new file: `crates/ralph-core/src/session.rs`
2. Define `SessionStatus` enum with variants: Planning, InProgress, Completed, Failed, Abandoned
3. Define `Session` struct with fields:
   - `id: String` - Full session ID (e.g., "001-rest-api")
   - `number: u32` - Numeric portion of the ID
   - `title: String` - Human-readable title
   - `path: PathBuf` - Absolute path to session directory
   - `status: SessionStatus` - Current lifecycle state
   - `created_at: DateTime<Utc>` - Creation timestamp
4. Implement `Session::new(id: &str, path: PathBuf)` constructor
5. Implement `Session::number_from_id(id: &str) -> Option<u32>` helper to extract number from ID
6. Derive appropriate traits (Debug, Clone, PartialEq, Serialize, Deserialize)

## Dependencies
- `chrono` crate for DateTime handling (already in ralph-core dependencies)
- `serde` for serialization (already in ralph-core dependencies)
- Standard library `PathBuf` for paths

## Implementation Approach
1. Create the new `session.rs` file in `crates/ralph-core/src/`
2. Define `SessionStatus` enum first with all variants
3. Define `Session` struct with all required fields
4. Implement the `new()` constructor that parses the ID to extract the number
5. Implement `number_from_id()` as a static helper
6. Add unit tests for struct creation and ID parsing

## Acceptance Criteria

1. **SessionStatus Enum Definition**
   - Given the need to track session lifecycle
   - When defining the SessionStatus enum
   - Then it has variants: Planning, InProgress, Completed, Failed, Abandoned

2. **Session Struct Definition**
   - Given the need to represent a session
   - When defining the Session struct
   - Then it has fields: id, number, title, path, status, created_at

3. **Session Constructor**
   - Given a valid session ID "001-rest-api" and path
   - When calling Session::new()
   - Then a Session is created with number=1 extracted from the ID

4. **Number Extraction**
   - Given session ID "042-my-feature"
   - When calling Session::number_from_id()
   - Then it returns Some(42)

5. **Invalid ID Handling**
   - Given an invalid session ID without a number prefix
   - When calling Session::number_from_id()
   - Then it returns None

6. **Unit Test Coverage**
   - Given the Session implementation
   - When running `cargo test -p ralph-core session`
   - Then all tests pass covering struct creation and ID parsing

## Metadata
- **Complexity**: Low
- **Labels**: Core, Session, Data Model
- **Required Skills**: Rust structs, enums, chrono, serde

---
status: completed
created: 2026-01-20
started: 2026-01-20
completed: 2026-01-20
---
# Task: Implement Path Helper Methods

## Description
Add path helper methods to the `Session` struct that return the paths to all session artifacts (prompt, scratchpad, events, summary) and subdirectories (plan, tasks, implementation).

## Background
Each session contains multiple artifacts stored in a consistent directory structure. Rather than constructing these paths ad-hoc throughout the codebase, the Session struct provides helper methods that guarantee consistent path construction. This follows the pattern established in the detailed design where the session directory structure is:

```
.ralph-o/sessions/001-rest-api/
├── PROMPT.md
├── scratchpad.md
├── events.jsonl
├── summary.md
├── plan/
├── tasks/
└── implementation/
```

## Reference Documentation
**Required:**
- Design: .sop/planning/design/detailed-design.md

**Note:** You MUST read the detailed design document before beginning implementation. It specifies the exact directory structure and file naming conventions.

## Technical Requirements
1. Add method `prompt_path(&self) -> PathBuf` returning `{session}/PROMPT.md`
2. Add method `scratchpad_path(&self) -> PathBuf` returning `{session}/scratchpad.md`
3. Add method `events_path(&self) -> PathBuf` returning `{session}/events.jsonl`
4. Add method `summary_path(&self) -> PathBuf` returning `{session}/summary.md`
5. Add method `plan_dir(&self) -> PathBuf` returning `{session}/plan/`
6. Add method `tasks_dir(&self) -> PathBuf` returning `{session}/tasks/`
7. Add method `implementation_dir(&self) -> PathBuf` returning `{session}/implementation/`
8. All methods should use `self.path.join()` for proper path construction

## Dependencies
- Task 1 (Session struct must exist)
- Standard library `PathBuf` for path operations

## Implementation Approach
1. Open `crates/ralph-core/src/session.rs`
2. Add an `impl Session` block (or extend existing one)
3. Implement each path helper method using `self.path.join("filename")`
4. Add unit tests that verify each method returns the expected path

## Acceptance Criteria

1. **Prompt Path**
   - Given a Session with path `/project/.ralph-o/sessions/001-test`
   - When calling `session.prompt_path()`
   - Then it returns `/project/.ralph-o/sessions/001-test/PROMPT.md`

2. **Scratchpad Path**
   - Given a Session with path `/project/.ralph-o/sessions/001-test`
   - When calling `session.scratchpad_path()`
   - Then it returns `/project/.ralph-o/sessions/001-test/scratchpad.md`

3. **Events Path**
   - Given a Session with path `/project/.ralph-o/sessions/001-test`
   - When calling `session.events_path()`
   - Then it returns `/project/.ralph-o/sessions/001-test/events.jsonl`

4. **Summary Path**
   - Given a Session with path `/project/.ralph-o/sessions/001-test`
   - When calling `session.summary_path()`
   - Then it returns `/project/.ralph-o/sessions/001-test/summary.md`

5. **Plan Directory**
   - Given a Session with path `/project/.ralph-o/sessions/001-test`
   - When calling `session.plan_dir()`
   - Then it returns `/project/.ralph-o/sessions/001-test/plan`

6. **Tasks Directory**
   - Given a Session with path `/project/.ralph-o/sessions/001-test`
   - When calling `session.tasks_dir()`
   - Then it returns `/project/.ralph-o/sessions/001-test/tasks`

7. **Implementation Directory**
   - Given a Session with path `/project/.ralph-o/sessions/001-test`
   - When calling `session.implementation_dir()`
   - Then it returns `/project/.ralph-o/sessions/001-test/implementation`

8. **Unit Test Coverage**
   - Given the path helper implementation
   - When running `cargo test -p ralph-core session`
   - Then all path helper tests pass

## Metadata
- **Complexity**: Low
- **Labels**: Core, Session, Paths
- **Required Skills**: Rust, PathBuf operations

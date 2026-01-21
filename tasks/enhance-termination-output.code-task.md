---
status: pending
created: 2026-01-21
started: null
completed: null
target_repo: null
bead: ralph-orchestrator-4a0
---
# Task: Enhance Termination Output with Summary Preview and File Links

## Description

After a `ralph-o run` completes, the termination box currently shows only iterations and elapsed time. Users want a quick summary and links to detailed artifacts without having to hunt for files.

## Background

The `SummaryWriter` already creates a detailed `summary.md` file with tasks, events, and commit info. The termination output should surface key stats from this file and tell users where to find it.

## Technical Requirements

1. Modify `print_termination()` in `crates/ralph-cli/src/main.rs` to accept session path
2. Read task stats from scratchpad (count `[x]`, `[ ]`, `[~]` markers)
3. Get commit count/range from git (commits since session start, or from summary)
4. Display paths to summary.md and scratchpad.md relative to project root
5. Handle missing files gracefully (session may not exist for non-session runs)

## Implementation Approach

1. **Locate `print_termination()`** - Around line 2660 in main.rs
2. **Add parameters** - Pass session path and scratchpad path
3. **Extract task stats** - Parse scratchpad for checkbox markers
4. **Extract commit info** - Either from summary.md or git log
5. **Update box rendering** - Add new rows for tasks, commits, and file paths
6. **Handle edge cases** - Non-session runs, missing files, no commits

## Current Code Location

```rust
// crates/ralph-cli/src/main.rs ~line 2660
fn print_termination(reason: &TerminationReason, state: &LoopState, use_colors: bool) {
    // Current implementation only shows iterations and elapsed
}
```

## Acceptance Criteria

1. **Task stats displayed**
   - Given a completed run with scratchpad
   - When termination box is printed
   - Then shows "Tasks: N completed, M pending, K cancelled"

2. **File paths displayed**
   - Given a session-based run
   - When termination box is printed  
   - Then shows relative paths to summary.md and scratchpad.md

3. **Graceful degradation**
   - Given a run without session (direct PROMPT.md)
   - When termination box is printed
   - Then shows only iterations/elapsed (current behavior)

4. **Color support maintained**
   - Given `--color=always` or `--color=never`
   - When termination box is printed
   - Then colors are applied/omitted correctly

5. **Non-TTY friendly**
   - Given output piped to file
   - When termination box is printed
   - Then box characters render correctly (or fall back to ASCII)

## Metadata
- **Complexity**: Low
- **Labels**: UX, CLI, termination, summary
- **Required Skills**: Rust, string formatting, file parsing

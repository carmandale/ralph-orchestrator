# Implement Session-Based Organization (Steps 2-16)

## Objective

Continue implementing the session-based organization feature for ralph-o. Step 1 (Session struct) is complete. Implement Steps 2-16 from the implementation plan.

## Reference Documents

- **Implementation Plan**: `.sop/planning/implementation/plan.md`
- **Detailed Design**: `.sop/planning/design/detailed-design.md`
- **Existing Code Tasks**: `tasks/step01/` (completed, for reference)

## Workflow

For each remaining step (2-16):

1. **Read the step** from `.sop/planning/implementation/plan.md`
2. **Generate code tasks** using `/code-task-generator .sop/planning/implementation/plan.md --step N`
3. **Implement each task** using `/code-assist tasks/stepNN/task-NN-*.code-task.md`
4. **Run tests**: `cargo test -p ralph-core` and `cargo test -p ralph-cli`
5. **Commit** when step is complete
6. **Update checklist** in plan.md to mark step complete

## Remaining Steps

- [ ] Step 2: Create SessionManager with create/list/get
- [ ] Step 3: Add current symlink management
- [ ] Step 4: Integrate Session paths with EventLogger/EventReader/SummaryWriter
- [ ] Step 5: Update `ralph init` for .ralph-o/config.yml
- [ ] Step 6: Update `ralph run` for session support
- [ ] Step 7: Update `ralph plan` with session context injection
- [ ] Step 8: Update `ralph task` with session context injection
- [ ] Step 9: Update `ralph resume` for session support
- [ ] Step 10: Add `ralph list` command
- [ ] Step 11: Update `ralph clean` for session support
- [ ] Step 12: Add migration logic for .agent/
- [ ] Step 13: Update PDD SOP for session-only output
- [ ] Step 14: Update code-task-generator SOP for session-only output
- [ ] Step 15: Update code-assist SOP for session-only output
- [ ] Step 16: Add smoke tests and documentation

## Backpressure

After each implementation:
```bash
cargo build
cargo test
cargo fmt --check
```

## Completion

Output `LOOP_COMPLETE` when all steps are implemented and tests pass.

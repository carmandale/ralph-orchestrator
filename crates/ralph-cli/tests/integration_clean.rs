use anyhow::Result;
use std::fs;
use std::os::unix::fs::symlink;
use std::process::Command;
use tempfile::TempDir;

/// Integration tests for clean command acceptance criteria.
///
/// Tests the `ralph clean` command which removes session directories.

#[test]
fn test_clean_basic_success() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();

    // Create .ralph-o/sessions directory
    let sessions_dir = temp_path.join(".ralph-o/sessions");
    fs::create_dir_all(&sessions_dir)?;

    // Create a session with files
    let session_dir = sessions_dir.join("001-test-session");
    fs::create_dir_all(&session_dir)?;
    fs::write(session_dir.join("PROMPT.md"), "test prompt")?;
    fs::write(session_dir.join("scratchpad.md"), "test content")?;
    fs::write(session_dir.join("events.jsonl"), "{}")?;

    // Set as current session
    let current_link = sessions_dir.join("current");
    symlink(&session_dir, &current_link)?;

    // Verify directory exists before cleanup
    assert!(session_dir.exists());
    assert!(session_dir.join("scratchpad.md").exists());

    // Run ralph clean (should clean current session)
    let output = Command::new(env!("CARGO_BIN_EXE_ralph"))
        .arg("clean")
        .current_dir(temp_path)
        .output()?;

    // Should succeed
    assert!(output.status.success(), "Command should succeed");

    // Session directory should be deleted
    assert!(!session_dir.exists(), "session directory should be deleted");

    // Output should contain success message
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("cleaned") || stdout.contains("Cleaned") || stdout.contains("✓"));

    Ok(())
}

#[test]
fn test_clean_with_session_flag() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();

    // Create .ralph-o/sessions directory
    let sessions_dir = temp_path.join(".ralph-o/sessions");
    fs::create_dir_all(&sessions_dir)?;

    // Create two sessions
    let session1 = sessions_dir.join("001-first-session");
    let session2 = sessions_dir.join("002-second-session");
    fs::create_dir_all(&session1)?;
    fs::create_dir_all(&session2)?;
    fs::write(session1.join("PROMPT.md"), "first prompt")?;
    fs::write(session2.join("PROMPT.md"), "second prompt")?;

    // Set session1 as current
    let current_link = sessions_dir.join("current");
    symlink(&session1, &current_link)?;

    // Verify both exist
    assert!(session1.exists());
    assert!(session2.exists());

    // Run ralph clean --session 002 (should clean session2, not current)
    let output = Command::new(env!("CARGO_BIN_EXE_ralph"))
        .arg("clean")
        .arg("--session")
        .arg("002")
        .current_dir(temp_path)
        .output()?;

    // Should succeed
    assert!(output.status.success());

    // Session 2 should be deleted, session 1 should remain
    assert!(session1.exists(), "session 1 should still exist");
    assert!(!session2.exists(), "session 2 should be deleted");

    Ok(())
}

#[test]
fn test_clean_dry_run() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();

    // Create .ralph-o/sessions directory
    let sessions_dir = temp_path.join(".ralph-o/sessions");
    fs::create_dir_all(&sessions_dir)?;

    // Create a session
    let session_dir = sessions_dir.join("001-test-session");
    fs::create_dir_all(&session_dir)?;
    fs::write(session_dir.join("PROMPT.md"), "test prompt")?;
    fs::write(session_dir.join("scratchpad.md"), "test")?;

    // Set as current
    let current_link = sessions_dir.join("current");
    symlink(&session_dir, &current_link)?;

    // Run ralph clean with --dry-run
    let output = Command::new(env!("CARGO_BIN_EXE_ralph"))
        .arg("clean")
        .arg("--dry-run")
        .current_dir(temp_path)
        .output()?;

    // Should succeed
    assert!(output.status.success());

    // Directory should still exist
    assert!(
        session_dir.exists(),
        "session directory should still exist after dry-run"
    );
    assert!(session_dir.join("scratchpad.md").exists());

    // Output should mention dry-run or preview
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Would delete") || stdout.contains("dry run") || stdout.contains("Dry run"),
        "Output should indicate dry-run mode"
    );

    Ok(())
}

#[test]
fn test_clean_no_current_session() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();

    // Create .ralph-o/sessions directory but no sessions
    let sessions_dir = temp_path.join(".ralph-o/sessions");
    fs::create_dir_all(&sessions_dir)?;

    // Run ralph clean (should fail - no current session)
    let output = Command::new(env!("CARGO_BIN_EXE_ralph"))
        .arg("clean")
        .current_dir(temp_path)
        .output()?;

    // Should fail since there's no current session
    assert!(
        !output.status.success(),
        "Command should fail when no current session exists"
    );

    // Output should indicate no current session
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("No current session") || stderr.contains("current session"),
        "Output should indicate no current session"
    );

    Ok(())
}

#[test]
fn test_clean_color_output_never() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();

    // Create .ralph-o/sessions directory
    let sessions_dir = temp_path.join(".ralph-o/sessions");
    fs::create_dir_all(&sessions_dir)?;

    // Create a session
    let session_dir = sessions_dir.join("001-test-session");
    fs::create_dir_all(&session_dir)?;
    fs::write(session_dir.join("PROMPT.md"), "test prompt")?;
    fs::write(session_dir.join("scratchpad.md"), "test")?;

    // Set as current
    let current_link = sessions_dir.join("current");
    symlink(&session_dir, &current_link)?;

    // Run ralph clean with --color never
    let output = Command::new(env!("CARGO_BIN_EXE_ralph"))
        .arg("clean")
        .arg("--color")
        .arg("never")
        .current_dir(temp_path)
        .output()?;

    assert!(output.status.success());

    // Output should not contain ANSI color codes
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("\x1b["),
        "Output should not contain ANSI escape codes when --color never is used"
    );

    Ok(())
}

#[test]
fn test_clean_color_output_always() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();

    // Create .ralph-o/sessions directory
    let sessions_dir = temp_path.join(".ralph-o/sessions");
    fs::create_dir_all(&sessions_dir)?;

    // Create a session
    let session_dir = sessions_dir.join("001-test-session");
    fs::create_dir_all(&session_dir)?;
    fs::write(session_dir.join("PROMPT.md"), "test prompt")?;
    fs::write(session_dir.join("scratchpad.md"), "test")?;

    // Set as current
    let current_link = sessions_dir.join("current");
    symlink(&session_dir, &current_link)?;

    // Run ralph clean with --color always
    let output = Command::new(env!("CARGO_BIN_EXE_ralph"))
        .arg("clean")
        .arg("--color")
        .arg("always")
        .current_dir(temp_path)
        .output()?;

    assert!(output.status.success());

    // Output should contain ANSI color codes
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("\x1b["),
        "Output should contain ANSI escape codes when --color always is used"
    );

    Ok(())
}

#[test]
#[cfg(unix)]
fn test_clean_permission_error() -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();

    // Create .ralph-o/sessions directory
    let sessions_dir = temp_path.join(".ralph-o/sessions");
    fs::create_dir_all(&sessions_dir)?;

    // Create a session
    let session_dir = sessions_dir.join("001-test-session");
    fs::create_dir_all(&session_dir)?;
    fs::write(session_dir.join("PROMPT.md"), "test prompt")?;
    fs::write(session_dir.join("scratchpad.md"), "test")?;

    // Set as current
    let current_link = sessions_dir.join("current");
    symlink(&session_dir, &current_link)?;

    // Make directory read-only (remove write permissions)
    let mut perms = fs::metadata(&session_dir)?.permissions();
    perms.set_mode(0o444); // Read-only
    fs::set_permissions(&session_dir, perms)?;

    // Run ralph clean
    let output = Command::new(env!("CARGO_BIN_EXE_ralph"))
        .arg("clean")
        .current_dir(temp_path)
        .output()?;

    // Restore permissions for cleanup
    let mut perms = fs::metadata(&session_dir)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&session_dir, perms)?;

    // Should fail with non-zero exit code
    assert!(
        !output.status.success(),
        "Command should fail with permission error"
    );

    // Should contain error message
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("permission") || stderr.contains("Permission") || stderr.contains("Failed"),
        "Error message should mention permission issue"
    );

    Ok(())
}

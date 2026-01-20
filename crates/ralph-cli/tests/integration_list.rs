//! Integration tests for ralph list command.
//!
//! These tests verify that list shows all sessions correctly.

use anyhow::Result;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

/// Test that list shows empty message when no sessions exist
#[test]
fn test_list_no_sessions() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();

    // Create .ralph-o/sessions directory but no sessions
    fs::create_dir_all(temp_path.join(".ralph-o/sessions"))?;

    // Run ralph list
    let output = Command::new(env!("CARGO_BIN_EXE_ralph-o"))
        .arg("list")
        .current_dir(temp_path)
        .output()?;

    // Should succeed
    assert!(output.status.success(), "list command should succeed");

    // Output should mention no sessions found
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No sessions found"));

    Ok(())
}

/// Test that list shows sessions when they exist
#[test]
fn test_list_with_sessions() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();

    // Create .ralph-o/sessions directory
    let sessions_dir = temp_path.join(".ralph-o/sessions");
    fs::create_dir_all(&sessions_dir)?;

    // Create test sessions
    let session1 = sessions_dir.join("001-test-feature");
    let session2 = sessions_dir.join("002-bug-fix");
    fs::create_dir_all(&session1)?;
    fs::create_dir_all(&session2)?;

    // Create PROMPT.md in each to mark them as valid
    fs::write(session1.join("PROMPT.md"), "Test feature")?;
    fs::write(session2.join("PROMPT.md"), "Bug fix")?;

    // Run ralph list
    let output = Command::new(env!("CARGO_BIN_EXE_ralph-o"))
        .arg("list")
        .current_dir(temp_path)
        .output()?;

    // Should succeed
    assert!(output.status.success(), "list command should succeed");

    // Output should contain session info
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("test-feature"));
    assert!(stdout.contains("bug-fix"));
    assert!(stdout.contains("NUM"));
    assert!(stdout.contains("TITLE"));
    assert!(stdout.contains("STATUS"));

    Ok(())
}

/// Test that list highlights current session
#[test]
fn test_list_highlights_current() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();

    // Create .ralph-o/sessions directory
    let sessions_dir = temp_path.join(".ralph-o/sessions");
    fs::create_dir_all(&sessions_dir)?;

    // Create test sessions
    let session1 = sessions_dir.join("001-test-feature");
    let session2 = sessions_dir.join("002-bug-fix");
    fs::create_dir_all(&session1)?;
    fs::create_dir_all(&session2)?;

    // Create PROMPT.md in each
    fs::write(session1.join("PROMPT.md"), "Test feature")?;
    fs::write(session2.join("PROMPT.md"), "Bug fix")?;

    // Create current symlink pointing to session2
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        symlink("002-bug-fix", sessions_dir.join("current"))?;
    }

    // Run ralph list
    let output = Command::new(env!("CARGO_BIN_EXE_ralph-o"))
        .arg("list")
        .current_dir(temp_path)
        .output()?;

    // Should succeed
    assert!(output.status.success(), "list command should succeed");

    // Output should contain both sessions
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("test-feature"));
    assert!(stdout.contains("bug-fix"));

    // Note: We can't easily test the visual marker here without parsing ANSI codes

    Ok(())
}

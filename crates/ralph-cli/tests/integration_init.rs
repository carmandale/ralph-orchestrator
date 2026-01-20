//! Integration tests for ralph init command.
//!
//! These tests verify that init creates the correct directory structure
//! and config file in .ralph-o/

use anyhow::Result;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

/// Test that init --backend creates .ralph-o directory structure
#[test]
fn test_init_creates_ralph_o_directory() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();

    // Run ralph init --backend claude
    let output = Command::new(env!("CARGO_BIN_EXE_ralph-o"))
        .arg("init")
        .arg("--backend")
        .arg("claude")
        .current_dir(temp_path)
        .output()?;

    // Should succeed
    assert!(output.status.success(), "init command should succeed");

    // Verify .ralph-o/ directory exists
    assert!(temp_path.join(".ralph-o").exists());
    assert!(temp_path.join(".ralph-o").is_dir());

    // Verify .ralph-o/sessions/ directory exists
    assert!(temp_path.join(".ralph-o/sessions").exists());
    assert!(temp_path.join(".ralph-o/sessions").is_dir());

    // Verify .ralph-o/config.yml exists
    let config_path = temp_path.join(".ralph-o/config.yml");
    assert!(config_path.exists());
    assert!(config_path.is_file());

    // Verify config content
    let content = fs::read_to_string(&config_path)?;
    assert!(content.contains("backend: \"claude\""));
    assert!(content.contains("Ralph Orchestrator Configuration"));

    Ok(())
}

/// Test that init with preset creates correct structure
#[test]
fn test_init_from_preset_creates_directory() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();

    // Run ralph init --preset tdd-red-green
    let output = Command::new(env!("CARGO_BIN_EXE_ralph-o"))
        .arg("init")
        .arg("--preset")
        .arg("tdd-red-green")
        .current_dir(temp_path)
        .output()?;

    // Should succeed
    assert!(output.status.success(), "init command should succeed");

    // Verify directory structure
    assert!(temp_path.join(".ralph-o").exists());
    assert!(temp_path.join(".ralph-o/sessions").exists());
    assert!(temp_path.join(".ralph-o/config.yml").exists());

    // Verify config has preset content (preset may or may not have cli: section)
    let content = fs::read_to_string(temp_path.join(".ralph-o/config.yml"))?;
    // tdd-red-green preset should have event_loop and hats sections
    assert!(content.contains("event_loop:") || content.contains("hats:"));

    Ok(())
}

/// Test that init refuses to overwrite without --force
#[test]
fn test_init_refuses_overwrite_without_force() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();

    // Run init first time
    let output1 = Command::new(env!("CARGO_BIN_EXE_ralph-o"))
        .arg("init")
        .arg("--backend")
        .arg("claude")
        .current_dir(temp_path)
        .output()?;
    assert!(output1.status.success());

    // Try to run init again without force - should fail
    let output2 = Command::new(env!("CARGO_BIN_EXE_ralph-o"))
        .arg("init")
        .arg("--backend")
        .arg("kiro")
        .current_dir(temp_path)
        .output()?;

    // Should fail
    assert!(!output2.status.success(), "init should fail without --force");

    // Error message should mention existing file
    let stderr = String::from_utf8_lossy(&output2.stderr);
    assert!(
        stderr.contains(".ralph-o/config.yml already exists")
            || stderr.contains("already exists"),
        "stderr: {}",
        stderr
    );

    Ok(())
}

/// Test that init with --force overwrites existing config
#[test]
fn test_init_with_force_overwrites() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();

    // Run init first time with claude
    let output1 = Command::new(env!("CARGO_BIN_EXE_ralph-o"))
        .arg("init")
        .arg("--backend")
        .arg("claude")
        .current_dir(temp_path)
        .output()?;
    assert!(output1.status.success());

    let content1 = fs::read_to_string(temp_path.join(".ralph-o/config.yml"))?;
    assert!(content1.contains("backend: \"claude\""));

    // Run init again with --force and different backend
    let output2 = Command::new(env!("CARGO_BIN_EXE_ralph-o"))
        .arg("init")
        .arg("--backend")
        .arg("kiro")
        .arg("--force")
        .current_dir(temp_path)
        .output()?;
    assert!(output2.status.success());

    let content2 = fs::read_to_string(temp_path.join(".ralph-o/config.yml"))?;
    assert!(content2.contains("backend: \"kiro\""));
    assert!(!content2.contains("backend: \"claude\""));

    Ok(())
}

/// Test that sessions directory is created if missing
#[test]
fn test_init_creates_sessions_dir_if_missing() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();

    // Create .ralph-o/ manually but not sessions/
    fs::create_dir(temp_path.join(".ralph-o"))?;

    // Run init
    let output = Command::new(env!("CARGO_BIN_EXE_ralph-o"))
        .arg("init")
        .arg("--backend")
        .arg("claude")
        .current_dir(temp_path)
        .output()?;

    assert!(output.status.success());

    // Verify sessions/ was created
    assert!(temp_path.join(".ralph-o/sessions").exists());
    assert!(temp_path.join(".ralph-o/sessions").is_dir());

    Ok(())
}

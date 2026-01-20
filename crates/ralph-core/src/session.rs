//! Session management for ralph-o.
//!
//! This module provides the core `Session` struct that represents a single
//! ralph-o session, along with `SessionStatus` to track the session lifecycle.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Session lifecycle status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    /// Session created via `ralph plan`, PDD workflow in progress.
    Planning,
    /// Execution started, not yet complete.
    InProgress,
    /// Terminated with completion promise.
    Completed,
    /// Terminated due to error/safeguard.
    Failed,
    /// User explicitly abandoned.
    Abandoned,
}

/// Represents a single ralph-o session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Session {
    /// Session ID (e.g., "001-rest-api").
    pub id: String,

    /// Session number (e.g., 1).
    pub number: u32,

    /// Session title (e.g., "rest-api").
    pub title: String,

    /// Root path to session directory.
    pub path: PathBuf,

    /// Session status.
    pub status: SessionStatus,

    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
}

impl Session {
    /// Creates a new Session from an ID and path.
    ///
    /// The ID must be in the format `NNN-title` where NNN is a zero-padded number.
    /// The title is extracted from everything after the first hyphen.
    /// Status defaults to `Planning` and created_at is set to now.
    pub fn new(id: &str, path: PathBuf) -> Self {
        let number = Self::number_from_id(id).unwrap_or(0);
        let title = id
            .find('-')
            .map(|i| id[i + 1..].to_string())
            .unwrap_or_default();

        Self {
            id: id.to_string(),
            number,
            title,
            path,
            status: SessionStatus::Planning,
            created_at: Utc::now(),
        }
    }

    /// Extracts the session number from an ID.
    ///
    /// The ID format is `NNN-title` where NNN is a zero-padded number.
    /// Returns `None` if the ID doesn't match the expected format.
    pub fn number_from_id(id: &str) -> Option<u32> {
        let hyphen_pos = id.find('-')?;
        let num_str = &id[..hyphen_pos];
        num_str.parse().ok()
    }

    // --- Path helpers ---

    /// Path to PROMPT.md.
    pub fn prompt_path(&self) -> PathBuf {
        self.path.join("PROMPT.md")
    }

    /// Path to scratchpad.md.
    pub fn scratchpad_path(&self) -> PathBuf {
        self.path.join("scratchpad.md")
    }

    /// Path to events.jsonl.
    pub fn events_path(&self) -> PathBuf {
        self.path.join("events.jsonl")
    }

    /// Path to summary.md.
    pub fn summary_path(&self) -> PathBuf {
        self.path.join("summary.md")
    }

    /// Path to plan/ directory (PDD artifacts).
    pub fn plan_dir(&self) -> PathBuf {
        self.path.join("plan")
    }

    /// Path to tasks/ directory (code task files).
    pub fn tasks_dir(&self) -> PathBuf {
        self.path.join("tasks")
    }

    /// Path to implementation/ directory (code-assist documentation).
    pub fn implementation_dir(&self) -> PathBuf {
        self.path.join("implementation")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_status_variants_exist() {
        // Verify all 5 variants are accessible
        let _planning = SessionStatus::Planning;
        let _in_progress = SessionStatus::InProgress;
        let _completed = SessionStatus::Completed;
        let _failed = SessionStatus::Failed;
        let _abandoned = SessionStatus::Abandoned;
    }

    #[test]
    fn session_status_derives_clone_and_eq() {
        let status = SessionStatus::Planning;
        let cloned = status.clone();
        assert_eq!(status, cloned);
    }

    #[test]
    fn session_status_derives_debug() {
        let status = SessionStatus::InProgress;
        let debug_str = format!("{:?}", status);
        assert!(debug_str.contains("InProgress"));
    }

    #[test]
    fn session_new_extracts_number() {
        let session = Session::new("001-rest-api", PathBuf::from("/tmp/sessions/001-rest-api"));
        assert_eq!(session.number, 1);
    }

    #[test]
    fn session_new_extracts_title() {
        let session = Session::new("001-rest-api", PathBuf::from("/tmp/sessions/001-rest-api"));
        assert_eq!(session.title, "rest-api");
    }

    #[test]
    fn session_new_sets_id() {
        let session = Session::new("001-rest-api", PathBuf::from("/tmp/sessions/001-rest-api"));
        assert_eq!(session.id, "001-rest-api");
    }

    #[test]
    fn session_new_sets_path() {
        let path = PathBuf::from("/tmp/sessions/001-rest-api");
        let session = Session::new("001-rest-api", path.clone());
        assert_eq!(session.path, path);
    }

    #[test]
    fn session_new_defaults_to_planning_status() {
        let session = Session::new("001-rest-api", PathBuf::from("/tmp"));
        assert_eq!(session.status, SessionStatus::Planning);
    }

    #[test]
    fn session_new_sets_created_at() {
        let before = Utc::now();
        let session = Session::new("001-rest-api", PathBuf::from("/tmp"));
        let after = Utc::now();

        assert!(session.created_at >= before);
        assert!(session.created_at <= after);
    }

    #[test]
    fn number_from_id_valid_single_digit() {
        assert_eq!(Session::number_from_id("001-rest-api"), Some(1));
    }

    #[test]
    fn number_from_id_valid_double_digit() {
        assert_eq!(Session::number_from_id("042-my-feature"), Some(42));
    }

    #[test]
    fn number_from_id_valid_triple_digit() {
        assert_eq!(Session::number_from_id("100-test"), Some(100));
    }

    #[test]
    fn number_from_id_invalid_no_hyphen() {
        assert_eq!(Session::number_from_id("invalid"), None);
    }

    #[test]
    fn number_from_id_invalid_empty() {
        assert_eq!(Session::number_from_id(""), None);
    }

    #[test]
    fn number_from_id_invalid_no_number() {
        assert_eq!(Session::number_from_id("abc-test"), None);
    }

    #[test]
    fn number_from_id_invalid_starts_with_hyphen() {
        assert_eq!(Session::number_from_id("-test"), None);
    }

    #[test]
    fn session_serialization_roundtrip() {
        let session = Session::new("001-rest-api", PathBuf::from("/tmp/sessions/001-rest-api"));
        let json = serde_json::to_string(&session).expect("serialize");
        let deserialized: Session = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(session.id, deserialized.id);
        assert_eq!(session.number, deserialized.number);
        assert_eq!(session.title, deserialized.title);
        assert_eq!(session.path, deserialized.path);
        assert_eq!(session.status, deserialized.status);
        assert_eq!(session.created_at, deserialized.created_at);
    }

    #[test]
    fn session_status_serializes_as_snake_case() {
        let status = SessionStatus::InProgress;
        let json = serde_json::to_string(&status).expect("serialize");
        assert_eq!(json, "\"in_progress\"");
    }

    #[test]
    fn session_derives_debug() {
        let session = Session::new("001-test", PathBuf::from("/tmp"));
        let debug_str = format!("{:?}", session);
        assert!(debug_str.contains("Session"));
        assert!(debug_str.contains("001-test"));
    }

    #[test]
    fn session_title_with_multiple_hyphens() {
        let session = Session::new("001-my-long-feature-name", PathBuf::from("/tmp"));
        assert_eq!(session.title, "my-long-feature-name");
    }

    // --- Path helper tests ---

    #[test]
    fn prompt_path_returns_correct_path() {
        let session = Session::new(
            "001-test",
            PathBuf::from("/project/.ralph-o/sessions/001-test"),
        );
        assert_eq!(
            session.prompt_path(),
            PathBuf::from("/project/.ralph-o/sessions/001-test/PROMPT.md")
        );
    }

    #[test]
    fn scratchpad_path_returns_correct_path() {
        let session = Session::new(
            "001-test",
            PathBuf::from("/project/.ralph-o/sessions/001-test"),
        );
        assert_eq!(
            session.scratchpad_path(),
            PathBuf::from("/project/.ralph-o/sessions/001-test/scratchpad.md")
        );
    }

    #[test]
    fn events_path_returns_correct_path() {
        let session = Session::new(
            "001-test",
            PathBuf::from("/project/.ralph-o/sessions/001-test"),
        );
        assert_eq!(
            session.events_path(),
            PathBuf::from("/project/.ralph-o/sessions/001-test/events.jsonl")
        );
    }

    #[test]
    fn summary_path_returns_correct_path() {
        let session = Session::new(
            "001-test",
            PathBuf::from("/project/.ralph-o/sessions/001-test"),
        );
        assert_eq!(
            session.summary_path(),
            PathBuf::from("/project/.ralph-o/sessions/001-test/summary.md")
        );
    }

    #[test]
    fn plan_dir_returns_correct_path() {
        let session = Session::new(
            "001-test",
            PathBuf::from("/project/.ralph-o/sessions/001-test"),
        );
        assert_eq!(
            session.plan_dir(),
            PathBuf::from("/project/.ralph-o/sessions/001-test/plan")
        );
    }

    #[test]
    fn tasks_dir_returns_correct_path() {
        let session = Session::new(
            "001-test",
            PathBuf::from("/project/.ralph-o/sessions/001-test"),
        );
        assert_eq!(
            session.tasks_dir(),
            PathBuf::from("/project/.ralph-o/sessions/001-test/tasks")
        );
    }

    #[test]
    fn implementation_dir_returns_correct_path() {
        let session = Session::new(
            "001-test",
            PathBuf::from("/project/.ralph-o/sessions/001-test"),
        );
        assert_eq!(
            session.implementation_dir(),
            PathBuf::from("/project/.ralph-o/sessions/001-test/implementation")
        );
    }
}

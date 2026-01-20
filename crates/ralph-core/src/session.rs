//! Session management for ralph-o.
//!
//! This module provides the core `Session` struct that represents a single
//! ralph-o session, along with `SessionStatus` to track the session lifecycle.
//! The `SessionManager` handles creating, listing, and retrieving sessions.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use thiserror::Error;

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

    // --- Status derivation ---

    /// Derives the session status from directory contents.
    ///
    /// Status is determined by checking for the presence of files:
    /// - If `summary.md` exists → parse to determine Completed vs Failed
    /// - If `events.jsonl` exists (but no summary) → InProgress
    /// - If `plan/` directory exists (but no events) → Planning
    /// - Otherwise → Planning (just created)
    pub fn derive_status(session_path: &Path) -> SessionStatus {
        let summary_path = session_path.join("summary.md");
        let events_path = session_path.join("events.jsonl");
        let plan_path = session_path.join("plan");

        if summary_path.exists() {
            // Parse summary to determine completed vs failed
            Self::parse_summary_status(&summary_path)
        } else if events_path.exists() {
            SessionStatus::InProgress
        } else if plan_path.exists() {
            SessionStatus::Planning
        } else {
            SessionStatus::Planning // Just created
        }
    }

    /// Parses a summary.md file to determine if the session completed or failed.
    ///
    /// Looks for failure indicators in the summary content.
    /// Returns `Failed` if the summary contains error indicators, otherwise `Completed`.
    fn parse_summary_status(summary_path: &Path) -> SessionStatus {
        if let Ok(content) = fs::read_to_string(summary_path) {
            let content_lower = content.to_lowercase();
            // Check for failure indicators in the summary
            if content_lower.contains("status: failed")
                || content_lower.contains("## failed")
                || content_lower.contains("termination: failed")
                || content_lower.contains("error:")
                    && (content_lower.contains("safeguard")
                        || content_lower.contains("max iterations"))
            {
                return SessionStatus::Failed;
            }
        }
        SessionStatus::Completed
    }

    /// Reads the last event from events.jsonl to check for error indicators.
    ///
    /// This is an alternative method for detecting failed status when
    /// no summary.md exists but the last event indicates failure.
    #[allow(dead_code)]
    fn check_events_for_failure(events_path: &Path) -> bool {
        if let Ok(file) = fs::File::open(events_path) {
            let reader = BufReader::new(file);
            let mut last_line = String::new();

            for line in reader.lines().map_while(Result::ok) {
                if !line.trim().is_empty() {
                    last_line = line;
                }
            }

            if !last_line.is_empty() {
                // Check if last event contains error indicators
                let line_lower = last_line.to_lowercase();
                return line_lower.contains("\"error\"")
                    || line_lower.contains("safeguard")
                    || line_lower.contains("max_iterations_reached");
            }
        }
        false
    }
}

/// Errors that can occur during session operations.
#[derive(Debug, Error)]
pub enum SessionError {
    /// Session not found.
    #[error("Session not found: {0}")]
    NotFound(String),

    /// Failed to create session directory.
    #[error("Failed to create session directory: {0}")]
    CreateFailed(String),

    /// Invalid session ID format.
    #[error("Invalid session ID format: {0}")]
    InvalidId(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Manages ralph-o sessions.
///
/// SessionManager handles creating, listing, and retrieving sessions
/// within a project's `.ralph-o/sessions/` directory.
pub struct SessionManager {
    /// Root path of the project.
    project_root: PathBuf,

    /// Path to the sessions directory (.ralph-o/sessions/).
    sessions_dir: PathBuf,
}

impl SessionManager {
    /// Creates a new SessionManager for the given project root.
    ///
    /// The sessions directory is `.ralph-o/sessions/` under the project root.
    pub fn new<P: AsRef<Path>>(project_root: P) -> Self {
        let project_root = project_root.as_ref().to_path_buf();
        let sessions_dir = project_root.join(".ralph-o").join("sessions");

        Self {
            project_root,
            sessions_dir,
        }
    }

    /// Creates a new session with the given title.
    ///
    /// This method:
    /// 1. Determines the next session number
    /// 2. Creates a session ID in the format `NNN-title`
    /// 3. Creates the session directory
    /// 4. Returns the new Session
    pub fn create(&self, title: &str) -> Result<Session, SessionError> {
        let number = self.next_number();
        let id = format!("{:03}-{}", number, Self::derive_title(title));
        let session_path = self.sessions_dir.join(&id);

        // Create session directory
        fs::create_dir_all(&session_path).map_err(|e| {
            SessionError::CreateFailed(format!("{}: {}", session_path.display(), e))
        })?;

        Ok(Session::new(&id, session_path))
    }

    /// Lists all sessions in the sessions directory.
    ///
    /// Sessions are returned sorted by session number (oldest first).
    pub fn list(&self) -> Result<Vec<Session>, SessionError> {
        if !self.sessions_dir.exists() {
            return Ok(Vec::new());
        }

        let mut sessions = Vec::new();

        for entry in fs::read_dir(&self.sessions_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                if let Some(id) = path.file_name().and_then(|n| n.to_str()) {
                    // Only include valid session IDs (NNN-title format)
                    if Session::number_from_id(id).is_some() {
                        let mut session = Session::new(id, path.clone());
                        // Derive status from directory contents
                        session.status = Session::derive_status(&path);
                        sessions.push(session);
                    }
                }
            }
        }

        // Sort by session number
        sessions.sort_by_key(|s| s.number);

        Ok(sessions)
    }

    /// Gets a session by ID or number.
    ///
    /// Accepts either a full session ID like "001-rest-api"
    /// or just a number like "1" or "001".
    pub fn get(&self, id_or_number: &str) -> Result<Session, SessionError> {
        // Try to parse as a number first
        if let Ok(number) = id_or_number.parse::<u32>() {
            // Find session by number
            let sessions = self.list()?;
            for session in sessions {
                if session.number == number {
                    return Ok(session);
                }
            }
            return Err(SessionError::NotFound(format!("Session number {}", number)));
        }

        // Otherwise treat as a full ID
        let session_path = self.sessions_dir.join(id_or_number);
        if !session_path.exists() {
            return Err(SessionError::NotFound(id_or_number.to_string()));
        }

        let mut session = Session::new(id_or_number, session_path.clone());
        session.status = Session::derive_status(&session_path);

        Ok(session)
    }

    /// Determines the next session number.
    ///
    /// This scans existing sessions and returns max + 1.
    /// Returns 1 if no sessions exist.
    fn next_number(&self) -> u32 {
        self.list()
            .ok()
            .and_then(|sessions| sessions.iter().map(|s| s.number).max())
            .map(|max| max + 1)
            .unwrap_or(1)
    }

    /// Derives a kebab-case title from the input string.
    ///
    /// This converts the title to lowercase and replaces spaces and
    /// underscores with hyphens.
    fn derive_title(title: &str) -> String {
        title
            .to_lowercase()
            .replace(' ', "-")
            .replace('_', "-")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-')
            .collect()
    }

    /// Gets the current session by reading the `current` symlink.
    ///
    /// Returns `None` if:
    /// - The symlink doesn't exist
    /// - The symlink points to a non-existent directory
    /// - The symlink is invalid
    pub fn current(&self) -> Result<Option<Session>, SessionError> {
        let current_link = self.sessions_dir.join("current");

        // Check if the symlink exists
        if !current_link.exists() {
            return Ok(None);
        }

        // Read the symlink target
        let target = fs::read_link(&current_link)?;

        // Get the session ID from the target path
        let session_id = target
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| SessionError::InvalidId("Invalid symlink target".to_string()))?;

        // Try to get the session
        match self.get(session_id) {
            Ok(session) => Ok(Some(session)),
            Err(SessionError::NotFound(_)) => {
                // Symlink points to deleted session
                Ok(None)
            }
            Err(e) => Err(e),
        }
    }

    /// Sets the current session by updating the `current` symlink.
    ///
    /// This removes any existing symlink and creates a new one pointing
    /// to the specified session.
    pub fn set_current(&self, session: &Session) -> Result<(), SessionError> {
        let current_link = self.sessions_dir.join("current");

        // Remove existing symlink if it exists
        if current_link.exists() || current_link.symlink_metadata().is_ok() {
            fs::remove_file(&current_link)?;
        }

        // Create new symlink (relative to sessions_dir)
        let target = PathBuf::from(&session.id);

        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&target, &current_link)?;
        }

        #[cfg(windows)]
        {
            std::os::windows::fs::symlink_dir(&target, &current_link)?;
        }

        Ok(())
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

    // --- derive_status tests ---

    #[test]
    fn derive_status_empty_dir_returns_planning() {
        let temp_dir = tempfile::tempdir().unwrap();
        let status = Session::derive_status(temp_dir.path());
        assert_eq!(status, SessionStatus::Planning);
    }

    #[test]
    fn derive_status_with_plan_dir_returns_planning() {
        let temp_dir = tempfile::tempdir().unwrap();
        fs::create_dir(temp_dir.path().join("plan")).unwrap();
        let status = Session::derive_status(temp_dir.path());
        assert_eq!(status, SessionStatus::Planning);
    }

    #[test]
    fn derive_status_with_events_returns_in_progress() {
        let temp_dir = tempfile::tempdir().unwrap();
        fs::write(temp_dir.path().join("events.jsonl"), "{}").unwrap();
        let status = Session::derive_status(temp_dir.path());
        assert_eq!(status, SessionStatus::InProgress);
    }

    #[test]
    fn derive_status_with_summary_returns_completed() {
        let temp_dir = tempfile::tempdir().unwrap();
        fs::write(
            temp_dir.path().join("summary.md"),
            "# Session Summary\n\nStatus: completed",
        )
        .unwrap();
        let status = Session::derive_status(temp_dir.path());
        assert_eq!(status, SessionStatus::Completed);
    }

    #[test]
    fn derive_status_with_failed_summary_returns_failed() {
        let temp_dir = tempfile::tempdir().unwrap();
        fs::write(
            temp_dir.path().join("summary.md"),
            "# Session Summary\n\nStatus: failed\n\nError: safeguard triggered",
        )
        .unwrap();
        let status = Session::derive_status(temp_dir.path());
        assert_eq!(status, SessionStatus::Failed);
    }

    #[test]
    fn derive_status_summary_takes_precedence_over_events() {
        let temp_dir = tempfile::tempdir().unwrap();
        fs::write(temp_dir.path().join("events.jsonl"), "{}").unwrap();
        fs::write(temp_dir.path().join("summary.md"), "# Summary").unwrap();
        let status = Session::derive_status(temp_dir.path());
        assert_eq!(status, SessionStatus::Completed);
    }

    #[test]
    fn derive_status_events_takes_precedence_over_plan() {
        let temp_dir = tempfile::tempdir().unwrap();
        fs::create_dir(temp_dir.path().join("plan")).unwrap();
        fs::write(temp_dir.path().join("events.jsonl"), "{}").unwrap();
        let status = Session::derive_status(temp_dir.path());
        assert_eq!(status, SessionStatus::InProgress);
    }

    #[test]
    fn check_events_for_failure_detects_error() {
        let temp_dir = tempfile::tempdir().unwrap();
        let events_path = temp_dir.path().join("events.jsonl");
        fs::write(&events_path, "{\"type\": \"error\"}").unwrap();
        assert!(Session::check_events_for_failure(&events_path));
    }

    #[test]
    fn check_events_for_failure_returns_false_for_normal_events() {
        let temp_dir = tempfile::tempdir().unwrap();
        let events_path = temp_dir.path().join("events.jsonl");
        fs::write(&events_path, "{\"type\": \"message\"}").unwrap();
        assert!(!Session::check_events_for_failure(&events_path));
    }

    // --- SessionManager tests ---

    #[test]
    fn session_manager_new_creates_manager() {
        let temp_dir = tempfile::tempdir().unwrap();
        let manager = SessionManager::new(temp_dir.path());
        assert_eq!(manager.project_root, temp_dir.path());
        assert_eq!(
            manager.sessions_dir,
            temp_dir.path().join(".ralph-o").join("sessions")
        );
    }

    #[test]
    fn session_manager_create_creates_first_session() {
        let temp_dir = tempfile::tempdir().unwrap();
        let manager = SessionManager::new(temp_dir.path());

        let session = manager.create("rest api").unwrap();

        assert_eq!(session.number, 1);
        assert_eq!(session.id, "001-rest-api");
        assert_eq!(session.title, "rest-api");
        assert!(session.path.exists());
    }

    #[test]
    fn session_manager_create_increments_session_number() {
        let temp_dir = tempfile::tempdir().unwrap();
        let manager = SessionManager::new(temp_dir.path());

        let session1 = manager.create("first session").unwrap();
        let session2 = manager.create("second session").unwrap();

        assert_eq!(session1.number, 1);
        assert_eq!(session2.number, 2);
        assert_eq!(session1.id, "001-first-session");
        assert_eq!(session2.id, "002-second-session");
    }

    #[test]
    fn session_manager_create_normalizes_title() {
        let temp_dir = tempfile::tempdir().unwrap();
        let manager = SessionManager::new(temp_dir.path());

        let session = manager.create("Add User Authentication").unwrap();

        assert_eq!(session.title, "add-user-authentication");
        assert_eq!(session.id, "001-add-user-authentication");
    }

    #[test]
    fn session_manager_list_empty_directory() {
        let temp_dir = tempfile::tempdir().unwrap();
        let manager = SessionManager::new(temp_dir.path());

        let sessions = manager.list().unwrap();
        assert_eq!(sessions.len(), 0);
    }

    #[test]
    fn session_manager_list_returns_sessions() {
        let temp_dir = tempfile::tempdir().unwrap();
        let manager = SessionManager::new(temp_dir.path());

        manager.create("first").unwrap();
        manager.create("second").unwrap();

        let sessions = manager.list().unwrap();
        assert_eq!(sessions.len(), 2);
        assert_eq!(sessions[0].number, 1);
        assert_eq!(sessions[1].number, 2);
    }

    #[test]
    fn session_manager_list_sorts_by_number() {
        let temp_dir = tempfile::tempdir().unwrap();
        let sessions_dir = temp_dir.path().join(".ralph-o").join("sessions");
        fs::create_dir_all(&sessions_dir).unwrap();

        // Create sessions out of order
        fs::create_dir(sessions_dir.join("003-third")).unwrap();
        fs::create_dir(sessions_dir.join("001-first")).unwrap();
        fs::create_dir(sessions_dir.join("002-second")).unwrap();

        let manager = SessionManager::new(temp_dir.path());
        let sessions = manager.list().unwrap();

        assert_eq!(sessions.len(), 3);
        assert_eq!(sessions[0].number, 1);
        assert_eq!(sessions[1].number, 2);
        assert_eq!(sessions[2].number, 3);
    }

    #[test]
    fn session_manager_list_ignores_invalid_dirs() {
        let temp_dir = tempfile::tempdir().unwrap();
        let sessions_dir = temp_dir.path().join(".ralph-o").join("sessions");
        fs::create_dir_all(&sessions_dir).unwrap();

        // Create a valid session
        fs::create_dir(sessions_dir.join("001-valid")).unwrap();
        // Create invalid directory names
        fs::create_dir(sessions_dir.join("invalid")).unwrap();
        fs::create_dir(sessions_dir.join("no-number")).unwrap();

        let manager = SessionManager::new(temp_dir.path());
        let sessions = manager.list().unwrap();

        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].id, "001-valid");
    }

    #[test]
    fn session_manager_list_derives_status() {
        let temp_dir = tempfile::tempdir().unwrap();
        let manager = SessionManager::new(temp_dir.path());

        let session = manager.create("test").unwrap();
        // Create events.jsonl to make status InProgress
        fs::write(session.events_path(), "{}").unwrap();

        let sessions = manager.list().unwrap();
        assert_eq!(sessions[0].status, SessionStatus::InProgress);
    }

    #[test]
    fn session_manager_get_by_number() {
        let temp_dir = tempfile::tempdir().unwrap();
        let manager = SessionManager::new(temp_dir.path());

        manager.create("first").unwrap();
        let created = manager.create("second").unwrap();

        let retrieved = manager.get("2").unwrap();
        assert_eq!(retrieved.id, created.id);
        assert_eq!(retrieved.number, 2);
    }

    #[test]
    fn session_manager_get_by_padded_number() {
        let temp_dir = tempfile::tempdir().unwrap();
        let manager = SessionManager::new(temp_dir.path());

        let created = manager.create("test").unwrap();

        let retrieved = manager.get("001").unwrap();
        assert_eq!(retrieved.id, created.id);
    }

    #[test]
    fn session_manager_get_by_id() {
        let temp_dir = tempfile::tempdir().unwrap();
        let manager = SessionManager::new(temp_dir.path());

        let created = manager.create("test session").unwrap();

        let retrieved = manager.get("001-test-session").unwrap();
        assert_eq!(retrieved.id, created.id);
    }

    #[test]
    fn session_manager_get_nonexistent_number() {
        let temp_dir = tempfile::tempdir().unwrap();
        let manager = SessionManager::new(temp_dir.path());

        let result = manager.get("999");
        assert!(result.is_err());
        assert!(matches!(result, Err(SessionError::NotFound(_))));
    }

    #[test]
    fn session_manager_get_nonexistent_id() {
        let temp_dir = tempfile::tempdir().unwrap();
        let manager = SessionManager::new(temp_dir.path());

        let result = manager.get("999-nonexistent");
        assert!(result.is_err());
        assert!(matches!(result, Err(SessionError::NotFound(_))));
    }

    #[test]
    fn session_manager_get_derives_status() {
        let temp_dir = tempfile::tempdir().unwrap();
        let manager = SessionManager::new(temp_dir.path());

        let session = manager.create("test").unwrap();
        // Create summary.md to make status Completed
        fs::write(session.summary_path(), "# Summary").unwrap();

        let retrieved = manager.get("1").unwrap();
        assert_eq!(retrieved.status, SessionStatus::Completed);
    }

    #[test]
    fn session_manager_next_number_starts_at_one() {
        let temp_dir = tempfile::tempdir().unwrap();
        let manager = SessionManager::new(temp_dir.path());

        assert_eq!(manager.next_number(), 1);
    }

    #[test]
    fn session_manager_next_number_increments() {
        let temp_dir = tempfile::tempdir().unwrap();
        let manager = SessionManager::new(temp_dir.path());

        manager.create("first").unwrap();
        assert_eq!(manager.next_number(), 2);

        manager.create("second").unwrap();
        assert_eq!(manager.next_number(), 3);
    }

    #[test]
    fn session_manager_derive_title_lowercase() {
        assert_eq!(
            SessionManager::derive_title("REST API"),
            "rest-api"
        );
    }

    #[test]
    fn session_manager_derive_title_spaces_to_hyphens() {
        assert_eq!(
            SessionManager::derive_title("add user auth"),
            "add-user-auth"
        );
    }

    #[test]
    fn session_manager_derive_title_underscores_to_hyphens() {
        assert_eq!(
            SessionManager::derive_title("fix_login_bug"),
            "fix-login-bug"
        );
    }

    #[test]
    fn session_manager_derive_title_removes_special_chars() {
        assert_eq!(
            SessionManager::derive_title("add! @user# $auth%"),
            "add-user-auth"
        );
    }

    #[test]
    fn session_manager_derive_title_preserves_hyphens() {
        assert_eq!(
            SessionManager::derive_title("auto-complete"),
            "auto-complete"
        );
    }

    // --- current/set_current tests ---

    #[test]
    fn session_manager_current_returns_none_when_no_symlink() {
        let temp_dir = tempfile::tempdir().unwrap();
        let manager = SessionManager::new(temp_dir.path());

        let current = manager.current().unwrap();
        assert!(current.is_none());
    }

    #[test]
    fn session_manager_set_current_creates_symlink() {
        let temp_dir = tempfile::tempdir().unwrap();
        let manager = SessionManager::new(temp_dir.path());

        let session = manager.create("test").unwrap();
        manager.set_current(&session).unwrap();

        let current_link = manager.sessions_dir.join("current");
        assert!(current_link.exists());
        assert!(current_link.symlink_metadata().unwrap().is_symlink());
    }

    #[test]
    fn session_manager_current_retrieves_set_session() {
        let temp_dir = tempfile::tempdir().unwrap();
        let manager = SessionManager::new(temp_dir.path());

        let session = manager.create("test").unwrap();
        manager.set_current(&session).unwrap();

        let current = manager.current().unwrap().unwrap();
        assert_eq!(current.id, session.id);
        assert_eq!(current.number, session.number);
    }

    #[test]
    fn session_manager_set_current_updates_existing_symlink() {
        let temp_dir = tempfile::tempdir().unwrap();
        let manager = SessionManager::new(temp_dir.path());

        let session1 = manager.create("first").unwrap();
        let session2 = manager.create("second").unwrap();

        // Set to first session
        manager.set_current(&session1).unwrap();
        let current = manager.current().unwrap().unwrap();
        assert_eq!(current.id, session1.id);

        // Update to second session
        manager.set_current(&session2).unwrap();
        let current = manager.current().unwrap().unwrap();
        assert_eq!(current.id, session2.id);
    }

    #[test]
    fn session_manager_current_returns_none_for_deleted_session() {
        let temp_dir = tempfile::tempdir().unwrap();
        let manager = SessionManager::new(temp_dir.path());

        let session = manager.create("test").unwrap();
        manager.set_current(&session).unwrap();

        // Delete the session directory
        fs::remove_dir_all(&session.path).unwrap();

        // current() should return None for deleted session
        let current = manager.current().unwrap();
        assert!(current.is_none());
    }

    #[test]
    fn session_manager_symlink_is_relative() {
        let temp_dir = tempfile::tempdir().unwrap();
        let manager = SessionManager::new(temp_dir.path());

        let session = manager.create("test").unwrap();
        manager.set_current(&session).unwrap();

        let current_link = manager.sessions_dir.join("current");
        let target = fs::read_link(&current_link).unwrap();

        // Target should be relative (just the session ID)
        assert_eq!(target, PathBuf::from("001-test"));
        assert!(!target.is_absolute());
    }

    #[test]
    fn session_manager_current_derives_status() {
        let temp_dir = tempfile::tempdir().unwrap();
        let manager = SessionManager::new(temp_dir.path());

        let session = manager.create("test").unwrap();
        // Add events.jsonl to make status InProgress
        fs::write(session.events_path(), "{}").unwrap();

        manager.set_current(&session).unwrap();

        let current = manager.current().unwrap().unwrap();
        assert_eq!(current.status, SessionStatus::InProgress);
    }
}

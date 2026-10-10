//! I/O-free automatic backup timing and restart policy.
use serde::{Deserialize, Serialize};

const DAY: u64 = 86_400;
const RETRY_DELAY: u64 = 3_600;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AutomaticBackupStatus {
    pub enabled: bool,
    pub interval_days: u32,
    pub running: bool,
    pub last_attempt_at: Option<u64>,
    pub last_success_at: Option<u64>,
    pub next_due_at: Option<u64>,
    pub last_error: Option<String>,
    pub last_snapshot: Option<String>,
}

impl Default for AutomaticBackupStatus {
    fn default() -> Self {
        Self {
            enabled: false,
            interval_days: 7,
            running: false,
            last_attempt_at: None,
            last_success_at: None,
            next_due_at: None,
            last_error: None,
            last_snapshot: None,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct BackupSchedule {
    pub status: AutomaticBackupStatus,
    pub target_key: String,
    pub retry_after: Option<u64>,
}

impl BackupSchedule {
    /// Call only when idle; a started job retains its own destination.
    pub fn configure(&mut self, now: u64, enabled: bool, days: u32, target: String) {
        if self.status.running {
            return;
        }
        if self.target_key != target {
            self.status = AutomaticBackupStatus::default();
            self.retry_after = None;
            self.target_key = target;
        }
        self.status.enabled = enabled;
        self.status.interval_days = days.clamp(1, 365);
        self.refresh_due(now);
    }

    fn refresh_due(&mut self, now: u64) {
        self.status.next_due_at = if self.status.enabled && !self.status.running {
            Some(
                self.status
                    .last_success_at
                    .map(|last| last.saturating_add(u64::from(self.status.interval_days) * DAY))
                    .unwrap_or(now)
                    .max(self.retry_after.unwrap_or(0)),
            )
        } else {
            None
        };
    }

    pub fn begin_if_due(&mut self, now: u64) -> bool {
        if !self.status.enabled
            || self.status.running
            || !self.status.next_due_at.is_some_and(|due| now >= due)
        {
            return false;
        }
        self.status.running = true;
        self.status.last_attempt_at = Some(now);
        self.status.next_due_at = None;
        true
    }

    pub fn finish(&mut self, now: u64, result: Result<String, String>) {
        if !self.status.running {
            return;
        }
        self.status.running = false;
        match result {
            Ok(filename) => {
                self.status.last_success_at = Some(now);
                self.status.last_snapshot = Some(filename);
                self.status.last_error = None;
                self.retry_after = None;
            }
            Err(error) => {
                self.status.last_error = Some(error);
                self.retry_after = Some(now.saturating_add(RETRY_DELAY));
            }
        }
        self.refresh_due(now);
    }

    /// A persisted in-flight attempt is never promoted to success after restart.
    pub fn recover_interrupted(&mut self, now: u64) {
        if self.status.running {
            self.finish(
                now,
                Err("Previous automatic backup was interrupted.".into()),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opt_in_due_dates_and_single_job_are_explicit() {
        let mut schedule = BackupSchedule::default();
        schedule.configure(100, false, 0, "destination".into());
        assert_eq!(schedule.status.interval_days, 1);
        assert!(!schedule.begin_if_due(100));
        schedule.configure(100, true, 7, "destination".into());
        assert!(!schedule.begin_if_due(99));
        assert!(schedule.begin_if_due(100));
        assert!(!schedule.begin_if_due(100));
        schedule.configure(101, false, 1, "other".into());
        assert_eq!(schedule.target_key, "destination");
        schedule.finish(110, Ok("snapshot.zip".into()));
        assert_eq!(schedule.status.next_due_at, Some(110 + 7 * DAY));
        assert!(!schedule.begin_if_due(90)); // clock rollback
        assert!(!schedule.begin_if_due(110 + 7 * DAY - 1));
        assert!(schedule.begin_if_due(110 + 7 * DAY));
    }

    #[test]
    fn failure_and_restart_retain_success_and_delay_retries() {
        let mut schedule = BackupSchedule::default();
        schedule.configure(100, true, 1, "destination".into());
        assert!(schedule.begin_if_due(100));
        schedule.finish(101, Ok("first.zip".into()));
        assert!(schedule.begin_if_due(101 + DAY));
        schedule.finish(102 + DAY, Err("provider unavailable".into()));
        assert_eq!(schedule.status.last_success_at, Some(101));
        assert_eq!(schedule.status.last_snapshot.as_deref(), Some("first.zip"));
        assert!(!schedule.begin_if_due(102 + DAY + RETRY_DELAY - 1));
        assert!(schedule.begin_if_due(102 + DAY + RETRY_DELAY));
        let mut restarted: BackupSchedule =
            serde_json::from_str(&serde_json::to_string(&schedule).unwrap()).unwrap();
        restarted.recover_interrupted(200 + DAY + RETRY_DELAY);
        assert!(!restarted.status.running);
        assert_eq!(restarted.status.last_success_at, Some(101));
        assert_eq!(
            restarted.status.next_due_at,
            Some(200 + DAY + 2 * RETRY_DELAY)
        );
        assert!(restarted.status.last_error.unwrap().contains("interrupted"));
    }

    #[test]
    fn disable_interval_changes_and_destination_reset_preserve_correct_history() {
        let mut schedule = BackupSchedule::default();
        schedule.configure(100, true, 7, "destination".into());
        schedule.begin_if_due(100);
        schedule.finish(101, Ok("first.zip".into()));
        schedule.configure(200, false, 900, "destination".into());
        assert_eq!(schedule.status.interval_days, 365);
        assert_eq!(schedule.status.next_due_at, None);
        assert_eq!(schedule.status.last_success_at, Some(101));
        schedule.configure(300, true, 1, "destination".into());
        assert_eq!(schedule.status.next_due_at, Some(101 + DAY));
        schedule.configure(300, true, 1, "new destination".into());
        assert_eq!(schedule.status.last_success_at, None);
        assert_eq!(schedule.status.last_snapshot, None);
        assert!(schedule.begin_if_due(300));
    }

    #[test]
    fn legacy_status_defaults_and_wire_fields_are_compatible() {
        let status: AutomaticBackupStatus = serde_json::from_str("{}").unwrap();
        assert_eq!(status, AutomaticBackupStatus::default());
        let value = serde_json::to_value(&status).unwrap();
        assert_eq!(value["interval_days"], 7);
        assert_eq!(value["enabled"], false);
        assert!(value["last_attempt_at"].is_null());
        assert!(value["next_due_at"].is_null());
        assert_eq!(value.as_object().unwrap().len(), 8);
    }
}

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskState {
    Backlog,
    Planned,
    Active,
    Done,
}

impl TaskState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Backlog => "backlog",
            Self::Planned => "planned",
            Self::Active => "active",
            Self::Done => "done",
        }
    }

    pub fn from_db(value: &str) -> Self {
        match value {
            "planned" => Self::Planned,
            "active" => Self::Active,
            "done" => Self::Done,
            _ => Self::Backlog,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskSource {
    Local,
    AzureDevOps,
}

impl TaskSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::AzureDevOps => "azure_devops",
        }
    }

    pub fn from_db(value: &str) -> Self {
        match value {
            "azure_devops" => Self::AzureDevOps,
            _ => Self::Local,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Task {
    pub id: i64,
    pub external_id: Option<String>,
    pub title: String,
    pub description: String,
    pub source: TaskSource,
    pub state: TaskState,
    pub estimated_minutes: Option<i32>,
    pub logged_minutes: i32,
    pub pending_remote_writeback_minutes: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScheduleKind {
    Task,
    Focus,
}

impl ScheduleKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Task => "task",
            Self::Focus => "focus",
        }
    }

    pub fn from_db(value: &str) -> Self {
        match value {
            "focus" => Self::Focus,
            _ => Self::Task,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScheduleBlock {
    pub id: i64,
    pub task_id: Option<i64>,
    pub title: String,
    pub kind: ScheduleKind,
    pub day: NaiveDate,
    pub start_minute: i32,
    pub end_minute: i32,
    pub notes: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CalendarEvent {
    pub id: i64,
    pub external_id: Option<String>,
    pub title: String,
    pub location: String,
    pub organizer: String,
    pub body: String,
    pub day: NaiveDate,
    pub start_minute: i32,
    pub end_minute: i32,
    pub source: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActivitySegment {
    pub id: i64,
    pub task_id: Option<i64>,
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
    pub process_name: String,
    pub exe_path: Option<String>,
    pub window_title: String,
    pub window_class: String,
    pub idle_seconds: i64,
    pub link_reason: String,
}

#[derive(Clone, Debug)]
pub struct TaskCompletionDraft {
    pub task_id: i64,
    pub title: String,
    pub minutes: i32,
    pub external_id: Option<String>,
}

impl TaskCompletionDraft {
    pub fn hours_label(&self) -> String {
        format!("{:.2}h", self.minutes as f32 / 60.0)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub azure_devops_org_url: String,
    pub azure_devops_project: String,
    pub azure_devops_pat: String,
    pub outlook_enabled: bool,
    pub azure_devops_enabled: bool,
    pub minimize_to_tray: bool,
    pub activity_poll_seconds: u64,
    pub idle_threshold_minutes: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            azure_devops_org_url: String::new(),
            azure_devops_project: String::new(),
            azure_devops_pat: String::new(),
            outlook_enabled: true,
            azure_devops_enabled: false,
            minimize_to_tray: true,
            activity_poll_seconds: 2,
            idle_threshold_minutes: 5,
        }
    }
}

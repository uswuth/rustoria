use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Todo {
    pub id: String,
    pub name: String,
    pub priority: Priority,
    pub tags: Vec<String>,
    pub status: TodoStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Priority {
    Low,
    Medium,
    High,
}

impl From<crate::cli::Priority> for Priority {
    fn from(p: crate::cli::Priority) -> Self {
        match p {
            crate::cli::Priority::Low => Priority::Low,
            crate::cli::Priority::Medium => Priority::Medium,
            crate::cli::Priority::High => Priority::High,
        }
    }
}

impl std::fmt::Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            Priority::Low => "Low",
            Priority::Medium => "Medium",
            Priority::High => "High",
        };
        f.write_str(label)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TodoStatus {
    Pending,
    InProgress,
    Completed,
}

impl From<crate::cli::Status> for TodoStatus {
    fn from(s: crate::cli::Status) -> Self {
        match s {
            crate::cli::Status::Pending => TodoStatus::Pending,
            crate::cli::Status::InProgress => TodoStatus::InProgress,
            crate::cli::Status::Completed => TodoStatus::Completed,
        }
    }
}

impl std::fmt::Display for TodoStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            TodoStatus::Pending => "Pending",
            TodoStatus::InProgress => "In Progress",
            TodoStatus::Completed => "Completed",
        };
        f.write_str(label)
    }
}

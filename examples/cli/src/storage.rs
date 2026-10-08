use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::models::Todo;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Storage {
    todos: Vec<Todo>,
}

impl Storage {
    pub fn load() -> Result<Self> {
        let path = Self::storage_path()?;
        tracing::debug!(path = %path.display(), "Loading storage");
        if !path.exists() {
            return Ok(Self::default());
        }
        let data = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read storage file: {}", path.display()))?;
        serde_json::from_str(&data).context("Failed to parse storage file")
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::storage_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }
        let data = serde_json::to_string_pretty(self)?;
        fs::write(&path, data)
            .with_context(|| format!("Failed to write storage file: {}", path.display()))?;
        tracing::debug!(path = %path.display(), todos = self.todos.len(), "Saved storage");
        Ok(())
    }

    pub fn save_todo(&mut self, todo: &Todo) -> Result<()> {
        self.todos.push(todo.clone());
        self.save()
    }

    pub fn list_todos(&self) -> Result<Vec<Todo>> {
        Ok(self.todos.clone())
    }

    pub fn delete_todo(&mut self, id: &str) -> Result<bool> {
        let len_before = self.todos.len();
        self.todos.retain(|t| t.id != id);
        let removed = self.todos.len() < len_before;
        if removed {
            self.save()?;
        }
        Ok(removed)
    }

    fn storage_path() -> Result<PathBuf> {
        if let Ok(path) = std::env::var("TODO_STORAGE_PATH") {
            return Ok(PathBuf::from(path));
        }
        let mut path = dirs::data_local_dir()
            .or_else(dirs::home_dir)
            .context("Could not determine storage directory")?;
        path.push("todo-cli");
        path.push("todos.json");
        Ok(path)
    }
}

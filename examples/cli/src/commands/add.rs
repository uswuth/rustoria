use crate::cli::Priority;
use crate::models::{Todo, TodoStatus};
use crate::storage::Storage;

pub fn execute(name: String, priority: Priority, tags: Vec<String>) -> anyhow::Result<()> {
    let mut storage = Storage::load()?;

    let todo = Todo {
        id: uuid::Uuid::new_v4().to_string(),
        name,
        priority: priority.into(),
        tags,
        status: TodoStatus::Pending,
        created_at: chrono::Utc::now(),
    };

    storage.save_todo(&todo)?;
    println!("Created todo: {} (ID: {})", todo.name, todo.id);

    Ok(())
}

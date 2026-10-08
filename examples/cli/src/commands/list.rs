use crate::cli::Status;
use crate::storage::Storage;

pub fn execute(status: Option<Status>, limit: usize) -> anyhow::Result<()> {
    let storage = Storage::load()?;
    let todos = storage.list_todos()?;

    let filtered: Vec<_> = todos
        .into_iter()
        .filter(|t| {
            status
                .as_ref()
                .map(|s| t.status == (*s).into())
                .unwrap_or(true)
        })
        .take(limit)
        .collect();

    if filtered.is_empty() {
        println!("No todos found.");
        return Ok(());
    }

    println!(
        "{:<36} {:<12} {:<10} {:<20} Tags",
        "ID", "Status", "Priority", "Name"
    );
    println!("{}", "-".repeat(90));

    for todo in filtered {
        let tags = if todo.tags.is_empty() {
            String::from("-")
        } else {
            todo.tags.join(", ")
        };
        println!(
            "{:<36} {:<12} {:<10} {:<20} {}",
            todo.id, todo.status, todo.priority, todo.name, tags
        );
    }

    Ok(())
}

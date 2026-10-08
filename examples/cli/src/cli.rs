use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "todo")]
#[command(about = "A simple todo list manager", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add a new todo item
    Add {
        /// The title of the todo item
        name: String,

        /// Priority level
        #[arg(short, long, value_enum, default_value = "medium")]
        priority: Priority,

        /// Tags for categorization (repeatable: `--tag work --tag urgent`)
        #[arg(short, long = "tag")]
        tags: Vec<String>,
    },
    /// List todo items
    List {
        /// Filter by status
        #[arg(short, long, value_enum)]
        status: Option<Status>,

        /// Limit number of results
        #[arg(short, long, default_value = "50")]
        limit: usize,
    },
    /// Remove a todo item by ID
    Remove {
        /// The ID of the todo item to remove
        id: String,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum Priority {
    Low,
    Medium,
    High,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum Status {
    Pending,
    InProgress,
    Completed,
}

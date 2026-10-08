use std::fs;
use std::path::PathBuf;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn setup_temp_storage() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().unwrap();
    let storage_path = temp_dir.path().join("todos.json");
    (temp_dir, storage_path)
}

#[test]
fn add_todo_and_list() {
    let (_temp, storage_path) = setup_temp_storage();

    // Add a todo
    let mut cmd = Command::cargo_bin("cli").unwrap();
    cmd.env("TODO_STORAGE_PATH", &storage_path)
        .arg("add")
        .arg("Buy groceries")
        .arg("--priority")
        .arg("high")
        .arg("--tag")
        .arg("shopping")
        .assert()
        .success()
        .stdout(predicate::str::contains("Created todo"));

    // List todos
    let mut cmd = Command::cargo_bin("cli").unwrap();
    cmd.env("TODO_STORAGE_PATH", &storage_path)
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("Buy groceries"))
        .stdout(predicate::str::contains("High"));
}

#[test]
fn add_multiple_todos_and_filter() {
    let (_temp, storage_path) = setup_temp_storage();

    // Add todos
    for (name, priority) in [("Task 1", "low"), ("Task 2", "high"), ("Task 3", "medium")] {
        let mut cmd = Command::cargo_bin("cli").unwrap();
        cmd.env("TODO_STORAGE_PATH", &storage_path)
            .arg("add")
            .arg(name)
            .arg("--priority")
            .arg(priority)
            .assert()
            .success();
    }

    // List all
    let mut cmd = Command::cargo_bin("cli").unwrap();
    cmd.env("TODO_STORAGE_PATH", &storage_path)
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("Task 1"))
        .stdout(predicate::str::contains("Task 2"))
        .stdout(predicate::str::contains("Task 3"));
}

#[test]
fn remove_todo() {
    let (_temp, storage_path) = setup_temp_storage();

    // Add a todo
    let mut cmd = Command::cargo_bin("cli").unwrap();
    cmd.env("TODO_STORAGE_PATH", &storage_path)
        .arg("add")
        .arg("To be removed")
        .assert()
        .success();

    // Get the ID from storage
    let storage_content = fs::read_to_string(&storage_path).unwrap();
    let storage: serde_json::Value = serde_json::from_str(&storage_content).unwrap();
    let id = storage["todos"][0]["id"].as_str().unwrap();

    // Remove the todo
    let mut cmd = Command::cargo_bin("cli").unwrap();
    cmd.env("TODO_STORAGE_PATH", &storage_path)
        .arg("remove")
        .arg(id)
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed todo"));

    // Verify it's gone
    let mut cmd = Command::cargo_bin("cli").unwrap();
    cmd.env("TODO_STORAGE_PATH", &storage_path)
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("No todos found"));
}

#[test]
fn remove_nonexistent_todo() {
    let (_temp, storage_path) = setup_temp_storage();

    let mut cmd = Command::cargo_bin("cli").unwrap();
    cmd.env("TODO_STORAGE_PATH", &storage_path)
        .arg("remove")
        .arg("nonexistent-id")
        .assert()
        .success()
        .stdout(predicate::str::contains("Todo not found"));
}

#[test]
fn list_empty() {
    let (_temp, storage_path) = setup_temp_storage();

    let mut cmd = Command::cargo_bin("cli").unwrap();
    cmd.env("TODO_STORAGE_PATH", &storage_path)
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("No todos found"));
}

#[test]
fn add_with_tags() {
    let (_temp, storage_path) = setup_temp_storage();

    let mut cmd = Command::cargo_bin("cli").unwrap();
    cmd.env("TODO_STORAGE_PATH", &storage_path)
        .arg("add")
        .arg("Tagged task")
        .arg("--tag")
        .arg("work")
        .arg("--tag")
        .arg("urgent")
        .assert()
        .success()
        .stdout(predicate::str::contains("Created todo"));
}

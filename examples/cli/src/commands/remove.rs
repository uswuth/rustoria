use crate::storage::Storage;

pub fn execute(id: String) -> anyhow::Result<()> {
    let mut storage = Storage::load()?;

    if storage.delete_todo(&id)? {
        println!("Removed todo: {}", id);
    } else {
        println!("Todo not found: {}", id);
    }

    Ok(())
}

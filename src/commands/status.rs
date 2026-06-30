use crate::storage::LocalStore;
use anyhow::Result;

pub fn run() -> Result<()> {
    let store = LocalStore::discover()?;
    let meta = store.read_meta()?;
    let ops = store.read_workspace_ops()?;
    println!("Canopy repository: {}", meta.name);
    println!("Format: {}", meta.format);
    match &meta.active_change {
        Some(handle) => {
            println!("Active change: change/{}", handle);
            let count = ops.ops.iter().filter(|op| &op.change == handle).count();
            println!("Workspace operations: {}", count);
        }
        None => println!("Active change: none"),
    }
    Ok(())
}

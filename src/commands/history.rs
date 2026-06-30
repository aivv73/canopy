use crate::{model::Projection, projection::projected_history, storage::LocalStore};
use anyhow::Result;

pub fn run(projection: Projection) -> Result<()> {
    let store = LocalStore::discover()?;
    let changes = projected_history(&store, projection)?;
    let shown = changes.len();
    println!("Projection history");
    println!("Projection: {}", projection);
    println!("History kind: accepted semantic deltas");
    for change in changes {
        println!();
        println!("Change: {}", change.name);
        println!("Handle: change/{}", change.handle);
        println!("Status: {}", change.status);
        println!(
            "Visibility: {}",
            match projection {
                Projection::Public => "public",
                Projection::Private => "private",
            }
        );
        if let Some(t) = change.visible_at {
            println!("Visible at: {}", t.to_rfc3339());
        }
        println!("Deltas:");
        for d in change.deltas {
            println!("  - {}", d.name);
        }
    }
    println!();
    println!("Changes shown: {}", shown);
    Ok(())
}

use crate::{
    model::{ChangeStatus, Projection},
    storage::LocalStore,
};
use anyhow::Result;

pub fn run(projection: Projection) -> Result<()> {
    let store = LocalStore::discover()?;
    let mut changes = store.load_changes()?;
    changes.sort_by_key(|c| c.created_at);
    for change in changes {
        if change.status != ChangeStatus::Accepted {
            continue;
        }
        if projection == Projection::Public
            && change.published_at.is_none()
            && change.disclosed_at.is_none()
        {
            continue;
        }
        let Some(proposal) = &change.proposal else {
            continue;
        };
        let visible: Vec<_> = proposal
            .semantic_deltas
            .iter()
            .filter(|d| projection == Projection::Private || d.class.public_safe())
            .collect();
        if visible.is_empty() {
            continue;
        }
        println!("Change: {}", change.name);
        println!("Handle: change/{}", change.handle);
        let shown_at = match projection {
            Projection::Public => change
                .disclosed_at
                .or(change.published_at)
                .or(change.accepted_at),
            Projection::Private => change.accepted_at,
        };
        if let Some(t) = shown_at {
            println!("Visible at: {}", t.to_rfc3339());
        }
        println!("Deltas:");
        for d in visible {
            println!("  - {}", d.name);
        }
    }
    Ok(())
}

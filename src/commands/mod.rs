//! Command orchestration for the `cnp` CLI.

mod change;
mod doctor;
mod file;
mod history;
mod status;

use crate::{
    cli::{ChangeCommand, Command, FileCommand, ProjectionCommand},
    model::PublicationMode,
};
use anyhow::Result;

pub fn run(command: Command) -> Result<()> {
    match command {
        Command::Init { path } => {
            let (_store, root) = crate::storage::init_layout(path)?;
            println!("Initialized Canopy MVP repository at {}", root.display());
            println!("Warning: this MVP is local-only and stores secret content in plaintext under .canopy/; projection filtering is not encryption.");
            Ok(())
        }
        Command::Change { command } => match command {
            ChangeCommand::Start { name } => change::start(&name),
            ChangeCommand::Correct {
                target_change,
                kind,
                name,
            } => change::correct(&target_change, kind, &name),
            ChangeCommand::List { all } => change::list(all),
            ChangeCommand::Show { change: change_ref } => change::show(&change_ref),
            ChangeCommand::Current => change::current(),
            ChangeCommand::Proposal { change: change_ref } => change::proposal_show(&change_ref),
            ChangeCommand::Finish { change: change_ref } => change::finish(&change_ref),
            ChangeCommand::Abandon { change: change_ref } => change::abandon(&change_ref),
            ChangeCommand::Propose { change: change_ref } => change::propose(&change_ref),
            ChangeCommand::Accept { change: change_ref } => change::accept(&change_ref),
            ChangeCommand::Publish {
                change: change_ref,
                to,
            } => change::publish(&change_ref, to, PublicationMode::Publish),
            ChangeCommand::Disclose {
                change: change_ref,
                to,
            } => change::publish(&change_ref, to, PublicationMode::Disclose),
        },
        Command::File { command } => match command {
            FileCommand::Add(args) => file::add(args),
            FileCommand::Update(args) => file::update(args),
            FileCommand::Remove(args) => file::remove(args),
            FileCommand::Rename(args) => file::rename(args),
        },
        Command::Status => status::run(),
        Command::Doctor => doctor::run(),
        Command::History { projection } => history::run(projection),
        Command::Projection { command } => match command {
            ProjectionCommand::Materialize {
                projection,
                out_dir,
            } => {
                let store = crate::storage::LocalStore::discover()?;
                let entries = crate::projection::materialized_entries(&store, projection)?;
                crate::materialize::materialize_entries(entries, &out_dir)?;
                println!(
                    "Materialized {} projection to {}",
                    projection,
                    out_dir.display()
                );
                Ok(())
            }
        },
    }
}

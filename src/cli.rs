use crate::model::{CorrectionKind, FileClass, Projection};
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "cnp", about = "Canopy MVP CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    Init {
        path: Option<PathBuf>,
    },
    Change {
        #[command(subcommand)]
        command: ChangeCommand,
    },
    File {
        #[command(subcommand)]
        command: FileCommand,
    },
    Status,
    Doctor,
    History {
        #[arg(long, value_enum)]
        projection: Projection,
    },
    Projection {
        #[command(subcommand)]
        command: ProjectionCommand,
    },
}

#[derive(Subcommand)]
pub enum ChangeCommand {
    Start {
        name: String,
    },
    Correct {
        target_change: String,
        #[arg(long, value_enum)]
        kind: CorrectionKind,
        #[arg(long)]
        name: String,
    },
    List {
        #[arg(long)]
        all: bool,
    },
    Show {
        change: String,
    },
    Current,
    Proposal {
        change: String,
    },
    Operations {
        change: String,
    },
    Finish {
        change: String,
    },
    Abandon {
        change: String,
    },
    Propose {
        change: String,
    },
    Accept {
        change: String,
    },
    Publish {
        change: String,
        #[arg(long, value_enum)]
        to: Projection,
    },
    Disclose {
        change: String,
        #[arg(long, value_enum)]
        to: Projection,
    },
}

#[derive(Subcommand)]
pub enum FileCommand {
    Add(FileAdd),
    Update(FileUpdate),
    Remove(FileRemove),
    Rename(FileRename),
}

#[derive(Args)]
pub struct FileAdd {
    pub path: PathBuf,
    #[arg(long = "class", value_enum, default_value_t = FileClass::PublicSource)]
    pub class: FileClass,
}

#[derive(Args)]
pub struct FileUpdate {
    pub path: PathBuf,
    #[arg(long = "class", value_enum)]
    pub class: Option<FileClass>,
}

#[derive(Args)]
pub struct FileRemove {
    pub path: PathBuf,
}

#[derive(Args)]
pub struct FileRename {
    pub old_path: PathBuf,
    pub new_path: PathBuf,
    #[arg(long = "class", value_enum)]
    pub class: Option<FileClass>,
}

#[derive(Subcommand)]
pub enum ProjectionCommand {
    Materialize {
        projection: Projection,
        out_dir: PathBuf,
    },
}

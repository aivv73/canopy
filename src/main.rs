//! Local-only `cnp` MVP CLI.
//!
//! This binary demonstrates Canopy's change-first workflow, promotion proposal
//! and acceptance, and public/private projection filtering using readable JSON
//! state under `.canopy/`.
//!
//! The MVP intentionally does not implement cryptographic privacy, remotes,
//! capability checks, live replicated workspaces, or durable schema migration.
//! Files classified as `secret` are filtered out of public projections but are
//! still stored in plaintext in `.canopy/`. See `docs/mvp.md` and `SECURITY.md`.

mod cli;
mod commands;
mod materialize;
mod model;
mod paths;
mod projection;
mod promotion;
mod storage;

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    commands::run(cli::Cli::parse().command)
}

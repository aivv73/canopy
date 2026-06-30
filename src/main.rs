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

use anyhow::{anyhow, bail, Context, Result};
use chrono::{DateTime, Utc};
use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

const CANOPY_DIR: &str = ".canopy";
const FORMAT: &str = "canopy-mvp-1";
const MATERIALIZATION_MARKER: &str = "canopy-mvp-materialization-v1\n";

#[derive(Parser)]
#[command(name = "cnp", about = "Canopy MVP CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
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
enum ChangeCommand {
    Start {
        name: String,
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
enum FileCommand {
    Add(FileAdd),
}

#[derive(Args)]
struct FileAdd {
    path: PathBuf,
    #[arg(long = "class", value_enum, default_value_t = FileClass::PublicSource)]
    class: FileClass,
}

#[derive(Subcommand)]
enum ProjectionCommand {
    Materialize {
        projection: Projection,
        out_dir: PathBuf,
    },
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, ValueEnum)]
#[serde(rename_all = "kebab-case")]
enum Projection {
    Public,
    Private,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, ValueEnum)]
#[serde(rename_all = "kebab-case")]
enum FileClass {
    PublicSource,
    ConfigTemplate,
    Secret,
}

impl FileClass {
    fn public_safe(&self) -> bool {
        !matches!(self, Self::Secret)
    }
}

impl std::fmt::Display for FileClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::PublicSource => "public-source",
            Self::ConfigTemplate => "config-template",
            Self::Secret => "secret",
        })
    }
}

impl std::fmt::Display for Projection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Public => "public",
            Self::Private => "private",
        })
    }
}

/// Temporary persisted JSON schema for `.canopy/repo.json`.
#[derive(Serialize, Deserialize)]
struct RepoMeta {
    name: String,
    format: String,
    active_change: Option<String>,
}

/// Temporary private full-tree cache for MVP materialization.
#[derive(Default, Serialize, Deserialize)]
struct VirtualTree {
    files: BTreeMap<String, FileEntry>,
}

#[derive(Serialize, Deserialize)]
struct FileEntry {
    content: String,
    class: FileClass,
    updated_at: DateTime<Utc>,
}

/// Temporary durable workspace operation log for the MVP.
#[derive(Default, Serialize, Deserialize)]
struct WorkspaceOps {
    ops: Vec<WorkspaceOp>,
}

#[derive(Clone, Serialize, Deserialize)]
struct WorkspaceOp {
    id: u64,
    change: String,
    kind: OpKind,
    path: String,
    content: String,
    class: FileClass,
    created_at: DateTime<Utc>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum OpKind {
    AddFile,
}

/// Temporary persisted change record under `.canopy/changes/`.
#[derive(Serialize, Deserialize)]
struct Change {
    name: String,
    handle: String,
    status: ChangeStatus,
    created_at: DateTime<Utc>,
    proposal: Option<PromotionProposal>,
    accepted_at: Option<DateTime<Utc>>,
    published_at: Option<DateTime<Utc>>,
    disclosed_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
enum ChangeStatus {
    Active,
    Proposed,
    Accepted,
}

#[derive(Serialize, Deserialize)]
struct PromotionProposal {
    semantic_deltas: Vec<SemanticDelta>,
    derived_from: Vec<u64>,
    proposed_at: DateTime<Utc>,
}

#[derive(Clone, Serialize, Deserialize)]
struct SemanticDelta {
    name: String,
    kind: OpKind,
    path: String,
    class: FileClass,
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Command::Init { path } => init(path),
        Command::Change { command } => match command {
            ChangeCommand::Start { name } => change_start(&name),
            ChangeCommand::Propose { change } => change_propose(&change),
            ChangeCommand::Accept { change } => change_accept(&change),
            ChangeCommand::Publish { change, to } => {
                change_publish(&change, to, PublicationMode::Publish)
            }
            ChangeCommand::Disclose { change, to } => {
                change_publish(&change, to, PublicationMode::Disclose)
            }
        },
        Command::File { command } => match command {
            FileCommand::Add(args) => file_add(args),
        },
        Command::History { projection } => history(projection),
        Command::Projection { command } => match command {
            ProjectionCommand::Materialize {
                projection,
                out_dir,
            } => materialize(projection, &out_dir),
        },
    }
}

fn init(path: Option<PathBuf>) -> Result<()> {
    let root = path.unwrap_or_else(|| PathBuf::from("."));
    if root.exists() && !root.is_dir() {
        bail!(
            "init path exists but is not a directory: {}",
            root.display()
        );
    }
    fs::create_dir_all(&root)?;
    let canopy = root.join(CANOPY_DIR);
    if canopy.exists() {
        bail!("Canopy repository already exists at {}", canopy.display());
    }
    fs::create_dir(&canopy)?;
    fs::create_dir(canopy.join("changes"))?;
    let name = root
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("canopy-repo")
        .to_string();
    write_json(
        &canopy.join("repo.json"),
        &RepoMeta {
            name,
            format: FORMAT.into(),
            active_change: None,
        },
    )?;
    write_json(&canopy.join("virtual-tree.json"), &VirtualTree::default())?;
    write_json(&canopy.join("workspace-ops.json"), &WorkspaceOps::default())?;
    println!("Initialized Canopy MVP repository at {}", root.display());
    println!("Warning: this MVP is local-only and stores secret content in plaintext under .canopy/; projection filtering is not encryption.");
    Ok(())
}

fn change_start(name: &str) -> Result<()> {
    let root = repo_root()?;
    let handle = slug(name);
    let path = change_path(&root, &handle);
    if path.exists() {
        bail!("change handle already exists: {}", handle);
    }
    let change = Change {
        name: name.into(),
        handle: handle.clone(),
        status: ChangeStatus::Active,
        created_at: Utc::now(),
        proposal: None,
        accepted_at: None,
        published_at: None,
        disclosed_at: None,
    };
    write_json(&path, &change)?;
    let mut meta: RepoMeta = read_json(&root.join("repo.json"))?;
    meta.active_change = Some(handle.clone());
    write_json(&root.join("repo.json"), &meta)?;
    println!("Started change: {}", name);
    println!("Handle: change/{}", handle);
    Ok(())
}

fn file_add(args: FileAdd) -> Result<()> {
    let root = repo_root()?;
    let meta: RepoMeta = read_json(&root.join("repo.json"))?;
    let change = meta
        .active_change
        .ok_or_else(|| anyhow!("no active change; run `cnp change start <name>` first"))?;
    let rel = normalize_rel(&args.path)?;
    let content =
        fs::read_to_string(&args.path).with_context(|| format!("read {}", args.path.display()))?;
    let now = Utc::now();
    let mut tree: VirtualTree = read_json(&root.join("virtual-tree.json"))?;
    tree.files.insert(
        rel.clone(),
        FileEntry {
            content: content.clone(),
            class: args.class.clone(),
            updated_at: now,
        },
    );
    write_json(&root.join("virtual-tree.json"), &tree)?;
    let mut ops: WorkspaceOps = read_json(&root.join("workspace-ops.json"))?;
    let id = ops.ops.iter().map(|op| op.id).max().unwrap_or(0) + 1;
    ops.ops.push(WorkspaceOp {
        id,
        change,
        kind: OpKind::AddFile,
        path: rel.clone(),
        content,
        class: args.class.clone(),
        created_at: now,
    });
    write_json(&root.join("workspace-ops.json"), &ops)?;
    println!("Added file: {}", rel);
    println!("Class: {}", args.class);
    if matches!(args.class, FileClass::Secret) {
        println!("Warning: Canopy MVP stores secret files in plaintext under .canopy/. They are hidden from public projections but not encrypted at rest.");
    }
    Ok(())
}

fn change_propose(change_ref: &str) -> Result<()> {
    let root = repo_root()?;
    let handle = resolve_change_handle(change_ref);
    let mut change: Change = read_json(&change_path(&root, &handle))?;
    let ops: WorkspaceOps = read_json(&root.join("workspace-ops.json"))?;
    let change_ops: Vec<_> = ops
        .ops
        .iter()
        .filter(|op| op.change == handle)
        .cloned()
        .collect();
    if change_ops.is_empty() {
        bail!("no workspace operations recorded for change/{}", handle);
    }
    let deltas = change_ops
        .iter()
        .map(|op| SemanticDelta {
            name: delta_name(op),
            kind: op.kind.clone(),
            path: op.path.clone(),
            class: op.class.clone(),
        })
        .collect();
    let derived_from = change_ops.iter().map(|op| op.id).collect();
    let proposal = PromotionProposal {
        semantic_deltas: deltas,
        derived_from,
        proposed_at: Utc::now(),
    };
    println!("Promotion proposal created for change: {}", change.name);
    for d in &proposal.semantic_deltas {
        println!("- {}", d.name);
    }
    change.proposal = Some(proposal);
    change.status = ChangeStatus::Proposed;
    write_json(&change_path(&root, &handle), &change)?;
    Ok(())
}

fn change_accept(change_ref: &str) -> Result<()> {
    let root = repo_root()?;
    let handle = resolve_change_handle(change_ref);
    let mut change: Change = read_json(&change_path(&root, &handle))?;
    if change.proposal.is_none() {
        bail!("change/{} has no promotion proposal", handle);
    }
    change.status = ChangeStatus::Accepted;
    change.accepted_at = Some(Utc::now());
    write_json(&change_path(&root, &handle), &change)?;
    println!("Accepted change: {}", change.name);
    println!("Handle: change/{}", handle);
    Ok(())
}

#[derive(Clone, Copy)]
enum PublicationMode {
    Publish,
    Disclose,
}

fn change_publish(change_ref: &str, to: Projection, mode: PublicationMode) -> Result<()> {
    if to != Projection::Public {
        bail!("MVP only supports publishing/disclosing to public");
    }
    let root = repo_root()?;
    let handle = resolve_change_handle(change_ref);
    let mut change: Change = read_json(&change_path(&root, &handle))?;
    if change.status != ChangeStatus::Accepted {
        bail!("change/{} must be accepted before publication", handle);
    }
    let now = Utc::now();
    match mode {
        PublicationMode::Disclose => {
            change.disclosed_at = Some(now);
            println!("Disclosed change to public: {}", change.name);
        }
        PublicationMode::Publish => {
            change.published_at = Some(now);
            println!("Published change to public: {}", change.name);
        }
    }
    write_json(&change_path(&root, &handle), &change)?;
    Ok(())
}

fn history(projection: Projection) -> Result<()> {
    let root = repo_root()?;
    let mut changes = load_changes(&root)?;
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

fn materialize(projection: Projection, out_dir: &Path) -> Result<()> {
    let root = repo_root()?;
    let tree: VirtualTree = read_json(&root.join("virtual-tree.json"))?;
    let visible_paths = materialized_paths(&root, projection)?;
    prepare_materialization_dir(out_dir)?;
    for path in visible_paths {
        let Some(entry) = tree.files.get(&path) else {
            continue;
        };
        let dest = safe_materialization_path(out_dir, &path)?;
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(dest, &entry.content)?;
    }
    fs::write(out_dir.join(".canopy-materialized"), MATERIALIZATION_MARKER)?;
    println!(
        "Materialized {} projection to {}",
        projection,
        out_dir.display()
    );
    Ok(())
}

fn materialized_paths(root: &Path, projection: Projection) -> Result<Vec<String>> {
    let tree: VirtualTree = read_json(&root.join("virtual-tree.json"))?;
    if projection == Projection::Private {
        return Ok(tree.files.keys().cloned().collect());
    }
    let mut paths = Vec::new();
    for change in load_changes(root)? {
        if change.status != ChangeStatus::Accepted
            || (change.published_at.is_none() && change.disclosed_at.is_none())
        {
            continue;
        }
        let Some(proposal) = change.proposal else {
            continue;
        };
        for delta in proposal.semantic_deltas {
            if delta.class.public_safe() && tree.files.contains_key(&delta.path) {
                paths.push(delta.path);
            }
        }
    }
    paths.sort();
    paths.dedup();
    Ok(paths)
}

fn prepare_materialization_dir(out_dir: &Path) -> Result<()> {
    if out_dir.exists() {
        if !out_dir.is_dir() {
            bail!(
                "materialization target exists but is not a directory: {}",
                out_dir.display()
            );
        }
        let marker = out_dir.join(".canopy-materialized");
        let mut entries = fs::read_dir(out_dir)?;
        if marker.exists() {
            let marker_content = fs::read_to_string(&marker)?;
            if marker_content != MATERIALIZATION_MARKER {
                bail!(
                    "materialization target has an invalid Canopy marker: {}",
                    out_dir.display()
                );
            }
            for entry in fs::read_dir(out_dir)? {
                let path = entry?.path();
                if path.is_dir() {
                    fs::remove_dir_all(path)?;
                } else {
                    fs::remove_file(path)?;
                }
            }
        } else if entries.next().is_some() {
            bail!(
                "materialization target is not empty and was not created by Canopy: {}",
                out_dir.display()
            );
        }
    }
    fs::create_dir_all(out_dir)?;
    Ok(())
}

fn safe_materialization_path(out_dir: &Path, virtual_path: &str) -> Result<PathBuf> {
    validate_virtual_path(virtual_path)?;
    Ok(out_dir.join(virtual_path))
}

fn repo_root() -> Result<PathBuf> {
    let cwd = std::env::current_dir()?;
    for dir in cwd.ancestors() {
        let c = dir.join(CANOPY_DIR);
        if c.is_dir() {
            return Ok(c);
        }
    }
    bail!("not inside a Canopy repository")
}

fn change_path(root: &Path, handle: &str) -> PathBuf {
    root.join("changes").join(format!("{}.json", handle))
}
fn resolve_change_handle(s: &str) -> String {
    let raw = s.strip_prefix("change/").unwrap_or(s);
    slug(raw)
}
fn slug(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}
fn delta_name(op: &WorkspaceOp) -> String {
    match op.kind {
        OpKind::AddFile => format!("add {}", op.path),
    }
}
fn normalize_rel(path: &Path) -> Result<String> {
    let path = path
        .to_string_lossy()
        .trim_start_matches("./")
        .replace('\\', "/");
    validate_virtual_path(&path)?;
    Ok(path)
}

fn validate_virtual_path(path: &str) -> Result<()> {
    let p = Path::new(path);
    if path.is_empty() || p.is_absolute() {
        bail!(
            "virtual paths must be non-empty repository-relative paths: {}",
            path
        );
    }
    for component in p.components() {
        use std::path::Component;
        match component {
            Component::Normal(part) if part != ".canopy" => {}
            _ => bail!("invalid virtual path: {}", path),
        }
    }
    Ok(())
}

fn load_changes(root: &Path) -> Result<Vec<Change>> {
    let mut out = vec![];
    for e in fs::read_dir(root.join("changes"))? {
        let e = e?;
        if e.path().extension().and_then(|s| s.to_str()) == Some("json") {
            out.push(read_json(&e.path())?);
        }
    }
    Ok(out)
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T> {
    Ok(serde_json::from_str(
        &fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?,
    )?)
}
fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, serde_json::to_string_pretty(value)? + "\n")
        .with_context(|| format!("write {}", tmp.display()))?;
    fs::rename(&tmp, path).with_context(|| format!("replace {}", path.display()))
}

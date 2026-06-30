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
enum ChangeCommand {
    Start {
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
enum FileCommand {
    Add(FileAdd),
    Update(FileUpdate),
    Remove(FileRemove),
    Rename(FileRename),
}

#[derive(Args)]
struct FileAdd {
    path: PathBuf,
    #[arg(long = "class", value_enum, default_value_t = FileClass::PublicSource)]
    class: FileClass,
}

#[derive(Args)]
struct FileUpdate {
    path: PathBuf,
    #[arg(long = "class", value_enum)]
    class: Option<FileClass>,
}

#[derive(Args)]
struct FileRemove {
    path: PathBuf,
}

#[derive(Args)]
struct FileRename {
    old_path: PathBuf,
    new_path: PathBuf,
    #[arg(long = "class", value_enum)]
    class: Option<FileClass>,
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
#[derive(Default, Serialize, Deserialize, PartialEq, Eq)]
struct VirtualTree {
    files: BTreeMap<String, FileEntry>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
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

/// A captured workspace operation.
///
/// Field contract by operation kind:
/// - add/update: `path`, `content`, and resulting `class` are present.
/// - remove: `path` and previous `class` are present; `content` is absent.
/// - rename: `path` is the old path, `new_path` is the new path, and `class`
///   is the resulting class; `content` is replayed from prior projection state.
#[derive(Clone, Serialize, Deserialize)]
struct WorkspaceOp {
    id: u64,
    change: String,
    kind: OpKind,
    path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    new_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    class: FileClass,
    created_at: DateTime<Utc>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum OpKind {
    #[serde(rename = "add-file")]
    Add,
    #[serde(rename = "update-file")]
    Update,
    #[serde(rename = "remove-file")]
    Remove,
    #[serde(rename = "rename-file")]
    Rename,
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

impl Change {
    fn has_accepted_or_visible_metadata(&self) -> bool {
        self.accepted_at.is_some() || self.published_at.is_some() || self.disclosed_at.is_some()
    }

    fn can_be_abandoned(&self) -> bool {
        matches!(self.status, ChangeStatus::Active | ChangeStatus::Proposed)
            && !self.has_accepted_or_visible_metadata()
    }
}

/// MVP lifecycle state persisted for a change.
///
/// `Abandoned` is terminal for unaccepted changes. Abandoned changes may retain
/// workspace operations and promotion proposals as intent history, but must not
/// have accepted, published, or disclosed lifecycle metadata and must not be the
/// repository's active change.
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
enum ChangeStatus {
    Active,
    Proposed,
    Accepted,
    Abandoned,
}

impl std::fmt::Display for ChangeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Active => "active",
            Self::Proposed => "proposed",
            Self::Accepted => "accepted",
            Self::Abandoned => "abandoned",
        })
    }
}

#[derive(Serialize, Deserialize)]
struct PromotionProposal {
    semantic_deltas: Vec<SemanticDelta>,
    derived_from: Vec<u64>,
    proposed_at: DateTime<Utc>,
}

/// A promoted semantic delta. The optional payload fields follow the same
/// operation-kind contract as `WorkspaceOp` and are persisted so public
/// materialization can replay published semantic state instead of reading the
/// current private virtual tree.
#[derive(Clone, Serialize, Deserialize)]
struct SemanticDelta {
    name: String,
    kind: OpKind,
    path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    new_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    class: FileClass,
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Command::Init { path } => init(path),
        Command::Change { command } => match command {
            ChangeCommand::Start { name } => change_start(&name),
            ChangeCommand::List { all } => change_list(all),
            ChangeCommand::Show { change } => change_show(&change),
            ChangeCommand::Current => change_current(),
            ChangeCommand::Proposal { change } => proposal_show(&change),
            ChangeCommand::Finish { change } => change_finish(&change),
            ChangeCommand::Abandon { change } => change_abandon(&change),
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
            FileCommand::Update(args) => file_update(args),
            FileCommand::Remove(args) => file_remove(args),
            FileCommand::Rename(args) => file_rename(args),
        },
        Command::Status => status(),
        Command::Doctor => doctor(),
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

fn status() -> Result<()> {
    let root = repo_root()?;
    let meta = read_repo_meta(&root)?;
    let ops: WorkspaceOps = read_json(&root.join("workspace-ops.json"))?;
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

fn change_list(all: bool) -> Result<()> {
    let root = repo_root()?;
    let mut changes = load_changes(&root)?;
    changes.sort_by_key(|c| c.created_at);
    for change in changes {
        if !all && change.status == ChangeStatus::Abandoned {
            continue;
        }
        println!(
            "{}\tchange/{}\t{}",
            change.name, change.handle, change.status
        );
    }
    Ok(())
}

fn change_current() -> Result<()> {
    let root = repo_root()?;
    let handle = active_change(&root)?;
    show_change_by_handle(&root, &handle)
}

fn change_show(change_ref: &str) -> Result<()> {
    let root = repo_root()?;
    let handle = resolve_change_handle(change_ref);
    show_change_by_handle(&root, &handle)
}

fn show_change_by_handle(root: &Path, handle: &str) -> Result<()> {
    let change: Change = read_json(&change_path(root, handle))?;
    println!("Change: {}", change.name);
    println!("Handle: change/{}", change.handle);
    println!("Status: {}", change.status);
    println!("Created at: {}", change.created_at.to_rfc3339());
    if let Some(t) = change.accepted_at {
        println!("Accepted at: {}", t.to_rfc3339());
    }
    if let Some(t) = change.published_at {
        println!("Published at: {}", t.to_rfc3339());
    }
    if let Some(t) = change.disclosed_at {
        println!("Disclosed at: {}", t.to_rfc3339());
    }
    if let Some(proposal) = &change.proposal {
        println!(
            "Promotion proposal: {} semantic deltas",
            proposal.semantic_deltas.len()
        );
        for delta in &proposal.semantic_deltas {
            println!("  - {}", delta.name);
        }
    }
    Ok(())
}

fn proposal_show(change_ref: &str) -> Result<()> {
    let root = repo_root()?;
    let handle = resolve_change_handle(change_ref);
    let change: Change = read_json(&change_path(&root, &handle))?;
    let Some(proposal) = change.proposal else {
        bail!("change/{} has no promotion proposal", handle);
    };
    println!("Promotion proposal for change: {}", change.name);
    println!("Proposed at: {}", proposal.proposed_at.to_rfc3339());
    println!(
        "Derived from workspace operations: {:?}",
        proposal.derived_from
    );
    println!("Semantic deltas:");
    for delta in proposal.semantic_deltas {
        println!("  - {}", delta.name);
    }
    Ok(())
}

fn change_finish(change_ref: &str) -> Result<()> {
    let root = repo_root()?;
    let handle = resolve_change_handle(change_ref);
    let mut meta = read_repo_meta(&root)?;
    let Some(active) = &meta.active_change else {
        bail!("no active change; run `cnp change start <name>` first");
    };
    if active != &handle {
        bail!(
            "cannot finish change/{} because change/{} is active",
            handle,
            active
        );
    }
    let change: Change = read_json(&change_path(&root, &handle))?;
    meta.active_change = None;
    write_json(&root.join("repo.json"), &meta)?;
    println!("Finished active change: {}", change.name);
    println!("Handle: change/{}", change.handle);
    println!("Active change: none");
    Ok(())
}

fn change_abandon(change_ref: &str) -> Result<()> {
    let root = repo_root()?;
    let handle = resolve_change_handle(change_ref);
    let path = change_path(&root, &handle);
    let mut change: Change = read_json(&path)?;
    if !change.can_be_abandoned() && change.status != ChangeStatus::Abandoned {
        bail!(
            "change/{} is accepted or visible and cannot be abandoned; use a future revert or supersede workflow",
            handle
        );
    }
    let was_abandoned = change.status == ChangeStatus::Abandoned;

    if !was_abandoned {
        change.status = ChangeStatus::Abandoned;
        write_json(&path, &change)?;
    }

    let mut meta = read_repo_meta(&root)?;
    if meta.active_change.as_deref() == Some(&handle) {
        meta.active_change = None;
        write_json(&root.join("repo.json"), &meta)?;
    }
    rebuild_private_virtual_tree(&root)?;

    if was_abandoned {
        println!("Change already abandoned: {}", change.name);
    } else {
        println!("Abandoned change: {}", change.name);
    }
    println!("Handle: change/{}", change.handle);
    if meta.active_change.is_none() {
        println!("Active change: none");
    }
    Ok(())
}

fn file_add(args: FileAdd) -> Result<()> {
    let root = repo_root()?;
    let change = active_change(&root)?;
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
    record_op(
        &root,
        WorkspaceOp {
            id: 0,
            change,
            kind: OpKind::Add,
            path: rel.clone(),
            new_path: None,
            content: Some(content),
            class: args.class.clone(),
            created_at: now,
        },
    )?;
    println!("Added file: {}", rel);
    println!("Class: {}", args.class);
    warn_secret(&args.class);
    Ok(())
}

fn file_update(args: FileUpdate) -> Result<()> {
    let root = repo_root()?;
    let change = active_change(&root)?;
    let rel = normalize_rel(&args.path)?;
    let content =
        fs::read_to_string(&args.path).with_context(|| format!("read {}", args.path.display()))?;
    let now = Utc::now();
    let mut tree: VirtualTree = read_json(&root.join("virtual-tree.json"))?;
    let existing = tree
        .files
        .get(&rel)
        .ok_or_else(|| anyhow!("cannot update unknown virtual file: {}", rel))?;
    let class = args.class.unwrap_or_else(|| existing.class.clone());
    if matches!(existing.class, FileClass::Secret) && class.public_safe() {
        bail!(
            "cannot reclassify secret file as public-safe during update: {}",
            rel
        );
    }
    tree.files.insert(
        rel.clone(),
        FileEntry {
            content: content.clone(),
            class: class.clone(),
            updated_at: now,
        },
    );
    write_json(&root.join("virtual-tree.json"), &tree)?;
    record_op(
        &root,
        WorkspaceOp {
            id: 0,
            change,
            kind: OpKind::Update,
            path: rel.clone(),
            new_path: None,
            content: Some(content),
            class: class.clone(),
            created_at: now,
        },
    )?;
    println!("Updated file: {}", rel);
    println!("Class: {}", class);
    warn_secret(&class);
    Ok(())
}

fn file_remove(args: FileRemove) -> Result<()> {
    let root = repo_root()?;
    let change = active_change(&root)?;
    let rel = normalize_rel(&args.path)?;
    let now = Utc::now();
    let mut tree: VirtualTree = read_json(&root.join("virtual-tree.json"))?;
    let removed = tree
        .files
        .remove(&rel)
        .ok_or_else(|| anyhow!("cannot remove unknown virtual file: {}", rel))?;
    write_json(&root.join("virtual-tree.json"), &tree)?;
    record_op(
        &root,
        WorkspaceOp {
            id: 0,
            change,
            kind: OpKind::Remove,
            path: rel.clone(),
            new_path: None,
            content: None,
            class: removed.class.clone(),
            created_at: now,
        },
    )?;
    println!("Removed file: {}", rel);
    println!("Class: {}", removed.class);
    warn_secret(&removed.class);
    Ok(())
}

fn file_rename(args: FileRename) -> Result<()> {
    let root = repo_root()?;
    let change = active_change(&root)?;
    let old = normalize_rel(&args.old_path)?;
    let new = normalize_rel(&args.new_path)?;
    if old == new {
        bail!("rename source and destination are the same: {}", old);
    }
    let now = Utc::now();
    let mut tree: VirtualTree = read_json(&root.join("virtual-tree.json"))?;
    if tree.files.contains_key(&new) {
        bail!("cannot rename over existing virtual file: {}", new);
    }
    let mut entry = tree
        .files
        .remove(&old)
        .ok_or_else(|| anyhow!("cannot rename unknown virtual file: {}", old))?;
    if matches!(entry.class, FileClass::Secret)
        && args.class.as_ref().is_some_and(FileClass::public_safe)
    {
        bail!(
            "cannot reclassify secret file as public-safe during rename: {}",
            old
        );
    }
    if let Some(class) = args.class {
        entry.class = class;
    }
    entry.updated_at = now;
    let class = entry.class.clone();
    tree.files.insert(new.clone(), entry);
    write_json(&root.join("virtual-tree.json"), &tree)?;
    record_op(
        &root,
        WorkspaceOp {
            id: 0,
            change,
            kind: OpKind::Rename,
            path: old.clone(),
            new_path: Some(new.clone()),
            content: None,
            class: class.clone(),
            created_at: now,
        },
    )?;
    println!("Renamed file: {} -> {}", old, new);
    println!("Class: {}", class);
    warn_secret(&class);
    Ok(())
}

fn active_change(root: &Path) -> Result<String> {
    let meta: RepoMeta = read_repo_meta(root)?;
    meta.active_change
        .ok_or_else(|| anyhow!("no active change; run `cnp change start <name>` first"))
}

fn record_op(root: &Path, mut op: WorkspaceOp) -> Result<()> {
    let mut ops: WorkspaceOps = read_json(&root.join("workspace-ops.json"))?;
    op.id = ops.ops.iter().map(|op| op.id).max().unwrap_or(0) + 1;
    ops.ops.push(op);
    write_json(&root.join("workspace-ops.json"), &ops)
}

fn warn_secret(class: &FileClass) {
    if matches!(class, FileClass::Secret) {
        println!("Warning: Canopy MVP stores secret files in plaintext under .canopy/. They are hidden from public projections but not encrypted at rest.");
    }
}

fn change_propose(change_ref: &str) -> Result<()> {
    let root = repo_root()?;
    let handle = resolve_change_handle(change_ref);
    let mut change: Change = read_json(&change_path(&root, &handle))?;
    if change.status == ChangeStatus::Abandoned {
        bail!("change/{} is abandoned and cannot be proposed", handle);
    }
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
            new_path: op.new_path.clone(),
            content: op.content.clone(),
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
    if change.status == ChangeStatus::Abandoned {
        bail!("change/{} is abandoned and cannot be accepted", handle);
    }
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

fn rebuild_private_virtual_tree(root: &Path) -> Result<()> {
    let tree = build_private_virtual_tree(root)?;
    write_json(&root.join("virtual-tree.json"), &tree)
}

fn build_private_virtual_tree(root: &Path) -> Result<VirtualTree> {
    // Canonical MVP replay for the private virtual-tree cache. Operations owned
    // by abandoned changes are retained as workspace history but intentionally
    // skipped so abandoned effects disappear from current private state.
    let changes = load_changes(root)?;
    let change_status: BTreeMap<_, _> = changes
        .into_iter()
        .map(|change| (change.handle, change.status))
        .collect();
    let ops: WorkspaceOps = read_json(&root.join("workspace-ops.json"))?;
    let mut tree = VirtualTree::default();

    for op in ops.ops {
        if change_status.get(&op.change) == Some(&ChangeStatus::Abandoned) {
            continue;
        }
        match op.kind {
            OpKind::Add | OpKind::Update => {
                let content = op.content.ok_or_else(|| {
                    anyhow!(
                        "malformed workspace operation {}: add/update missing content for {}",
                        op.id,
                        op.path
                    )
                })?;
                tree.files.insert(
                    op.path,
                    FileEntry {
                        content,
                        class: op.class,
                        updated_at: op.created_at,
                    },
                );
            }
            OpKind::Remove => {
                tree.files.remove(&op.path);
            }
            OpKind::Rename => {
                let new_path = op.new_path.ok_or_else(|| {
                    anyhow!(
                        "malformed workspace operation {}: rename missing new path for {}",
                        op.id,
                        op.path
                    )
                })?;
                if let Some(mut entry) = tree.files.remove(&op.path) {
                    entry.class = op.class;
                    entry.updated_at = op.created_at;
                    tree.files.insert(new_path, entry);
                }
            }
        }
    }
    Ok(tree)
}

fn materialize(projection: Projection, out_dir: &Path) -> Result<()> {
    let root = repo_root()?;
    let entries = materialized_entries(&root, projection)?;
    prepare_materialization_dir(out_dir)?;
    for (path, content) in entries {
        let dest = safe_materialization_path(out_dir, &path)?;
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(dest, content)?;
    }
    fs::write(out_dir.join(".canopy-materialized"), MATERIALIZATION_MARKER)?;
    println!(
        "Materialized {} projection to {}",
        projection,
        out_dir.display()
    );
    Ok(())
}

fn materialized_entries(root: &Path, projection: Projection) -> Result<BTreeMap<String, String>> {
    if projection == Projection::Private {
        let tree: VirtualTree = read_json(&root.join("virtual-tree.json"))?;
        return Ok(tree
            .files
            .into_iter()
            .map(|(path, entry)| (path, entry.content))
            .collect());
    }

    let mut entries = BTreeMap::new();
    let mut changes = load_changes(root)?;
    changes.sort_by_key(|c| c.created_at);
    for change in changes {
        if change.status != ChangeStatus::Accepted
            || (change.published_at.is_none() && change.disclosed_at.is_none())
        {
            continue;
        }
        let Some(proposal) = change.proposal else {
            continue;
        };
        for delta in proposal.semantic_deltas {
            if !delta.class.public_safe() {
                continue;
            }
            match delta.kind {
                OpKind::Add | OpKind::Update => {
                    if let Some(content) = delta.content {
                        entries.insert(delta.path, content);
                    }
                }
                OpKind::Remove => {
                    entries.remove(&delta.path);
                }
                OpKind::Rename => {
                    if let Some(new_path) = delta.new_path {
                        if let Some(content) = entries.remove(&delta.path) {
                            entries.insert(new_path, content);
                        }
                    }
                }
            }
        }
    }
    Ok(entries)
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

fn doctor() -> Result<()> {
    let root = repo_root()?;
    let mut errors = Vec::new();
    let mut warnings = vec![
        "MVP secret privacy is projection filtering over plaintext local JSON, not encryption"
            .to_string(),
    ];

    let mut active_handle = None;
    match read_repo_meta(&root) {
        Ok(meta) => {
            if let Some(active) = &meta.active_change {
                active_handle = Some(active.clone());
                if !change_path(&root, active).exists() {
                    errors.push(format!("active change does not exist: change/{}", active));
                }
            }
        }
        Err(e) => errors.push(e.to_string()),
    }

    let changes = match load_changes(&root) {
        Ok(changes) => changes,
        Err(e) => {
            errors.push(format!("cannot read changes: {e}"));
            Vec::new()
        }
    };
    let change_handles: Vec<_> = changes.iter().map(|c| c.handle.clone()).collect();
    if let Some(active) = active_handle {
        if let Some(change) = changes.iter().find(|c| c.handle == active) {
            if change.status == ChangeStatus::Abandoned {
                errors.push(format!(
                    "active change points to abandoned change/{}",
                    active
                ));
            }
            if change.status == ChangeStatus::Accepted {
                warnings.push(format!(
                    "accepted change is still active; run `cnp change finish change/{}` when editing is complete",
                    active
                ));
            }
            if change.published_at.is_some() || change.disclosed_at.is_some() {
                warnings.push(format!(
                    "published/disclosed change is still active; run `cnp change finish change/{}` when editing is complete",
                    active
                ));
            }
        }
    }
    for change in &changes {
        if change.status == ChangeStatus::Abandoned
            && (change.accepted_at.is_some()
                || change.published_at.is_some()
                || change.disclosed_at.is_some())
        {
            errors.push(format!(
                "abandoned change has accepted/published/disclosed metadata: change/{}",
                change.handle
            ));
        }
    }

    match read_json::<WorkspaceOps>(&root.join("workspace-ops.json")) {
        Ok(ops) => {
            for op in ops.ops {
                if !change_handles.contains(&op.change) {
                    errors.push(format!(
                        "workspace operation {} references missing change/{}",
                        op.id, op.change
                    ));
                }
                if let Err(e) = validate_virtual_path(&op.path) {
                    errors.push(format!(
                        "workspace operation {} has invalid path: {e}",
                        op.id
                    ));
                }
                if let Some(new_path) = &op.new_path {
                    if let Err(e) = validate_virtual_path(new_path) {
                        errors.push(format!(
                            "workspace operation {} has invalid new path: {e}",
                            op.id
                        ));
                    }
                }
            }
        }
        Err(e) => errors.push(format!("cannot read workspace operations: {e}")),
    }

    match read_json::<VirtualTree>(&root.join("virtual-tree.json")) {
        Ok(tree) => {
            for path in tree.files.keys() {
                if let Err(e) = validate_virtual_path(path) {
                    errors.push(format!("virtual tree has invalid path: {e}"));
                }
            }
            match build_private_virtual_tree(&root) {
                Ok(expected) if expected != tree => errors.push(
                    "virtual tree does not match replay of non-abandoned workspace operations"
                        .to_string(),
                ),
                Ok(_) => {}
                Err(e) => errors.push(format!("cannot replay private virtual tree: {e}")),
            }
        }
        Err(e) => errors.push(format!("cannot read virtual tree: {e}")),
    }

    println!("Canopy doctor");
    if errors.is_empty() {
        println!("Status: healthy");
    } else {
        println!("Status: errors found");
        for error in &errors {
            println!("Error: {}", error);
        }
    }
    for warning in warnings.drain(..) {
        println!("Warning: {}", warning);
    }
    if errors.is_empty() {
        Ok(())
    } else {
        bail!("doctor found {} error(s)", errors.len())
    }
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
        OpKind::Add => format!("add {}", op.path),
        OpKind::Update => format!("update {}", op.path),
        OpKind::Remove => format!("remove {}", op.path),
        OpKind::Rename => format!(
            "rename {} to {}",
            op.path,
            op.new_path.as_deref().unwrap_or("<missing>")
        ),
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

fn read_repo_meta(root: &Path) -> Result<RepoMeta> {
    let meta: RepoMeta = read_json(&root.join("repo.json"))?;
    if meta.format != FORMAT {
        bail!(
            "unsupported Canopy storage format `{}`; this cnp supports `{}`",
            meta.format,
            FORMAT
        );
    }
    Ok(meta)
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T> {
    let content = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&content).with_context(|| format!("parse JSON state {}", path.display()))
}
fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, serde_json::to_string_pretty(value)? + "\n")
        .with_context(|| format!("write {}", tmp.display()))?;
    fs::rename(&tmp, path).with_context(|| format!("replace {}", path.display()))
}

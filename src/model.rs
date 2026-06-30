use chrono::{DateTime, Utc};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum Projection {
    Public,
    Private,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum FileClass {
    PublicSource,
    ConfigTemplate,
    Secret,
}

impl FileClass {
    pub fn public_safe(&self) -> bool {
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

#[derive(Serialize, Deserialize)]
pub struct RepoMeta {
    pub name: String,
    pub format: String,
    pub active_change: Option<String>,
}

#[derive(Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct VirtualTree {
    pub files: BTreeMap<String, FileEntry>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
pub struct FileEntry {
    pub content: String,
    pub class: FileClass,
    pub updated_at: DateTime<Utc>,
}

#[derive(Default, Serialize, Deserialize)]
pub struct WorkspaceOps {
    pub ops: Vec<WorkspaceOp>,
}

/// A captured workspace operation.
///
/// Field contract by operation kind:
/// - add/update: `path`, `content`, and resulting `class` are present.
/// - remove: `path` and previous `class` are present; `content` is absent.
/// - rename: `path` is the old path, `new_path` is the new path, and `class`
///   is the resulting class; `content` is replayed from prior projection state.
#[derive(Clone, Serialize, Deserialize)]
pub struct WorkspaceOp {
    pub id: u64,
    pub change: String,
    pub kind: OpKind,
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub new_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    pub class: FileClass,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OpKind {
    #[serde(rename = "add-file")]
    Add,
    #[serde(rename = "update-file")]
    Update,
    #[serde(rename = "remove-file")]
    Remove,
    #[serde(rename = "rename-file")]
    Rename,
}

impl OpKind {
    pub fn inspection_label(&self) -> &'static str {
        match self {
            Self::Add => "add",
            Self::Update => "update",
            Self::Remove => "remove",
            Self::Rename => "rename",
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Change {
    pub name: String,
    pub handle: String,
    pub status: ChangeStatus,
    pub created_at: DateTime<Utc>,
    pub proposal: Option<PromotionProposal>,
    pub accepted_at: Option<DateTime<Utc>>,
    pub published_at: Option<DateTime<Utc>>,
    pub disclosed_at: Option<DateTime<Utc>>,
}

impl Change {
    pub fn has_accepted_or_visible_metadata(&self) -> bool {
        self.accepted_at.is_some() || self.published_at.is_some() || self.disclosed_at.is_some()
    }

    pub fn can_be_abandoned(&self) -> bool {
        matches!(self.status, ChangeStatus::Active | ChangeStatus::Proposed)
            && !self.has_accepted_or_visible_metadata()
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ChangeStatus {
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
pub struct PromotionProposal {
    pub semantic_deltas: Vec<SemanticDelta>,
    pub derived_from: Vec<u64>,
    pub proposed_at: DateTime<Utc>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SemanticDelta {
    pub name: String,
    pub kind: OpKind,
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub new_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    pub class: FileClass,
}

#[derive(Clone, Copy)]
pub enum PublicationMode {
    Publish,
    Disclose,
}

use serde::Serialize;

/// Declared configuration evidence, not an effective SDK/runtime timeout proof.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunnerConfigDeadlineEvidence {
    pub framework: String,
    pub status: RunnerDeadlineStatus,
    pub configs: Vec<ConfigDeadlineEvidence>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RunnerDeadlineStatus {
    NotRequested,
    Prepared,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum ConfigDeadlineEvidence {
    Prepared {
        config: String,
        projects: Vec<ProjectDeadlineEvidence>,
    },
    Failed {
        config: String,
        error: String,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDeadlineEvidence {
    pub config: Option<String>,
    pub workspace: bool,
    pub policy_name: Option<String>,
    pub runner_project_arg: Option<String>,
    pub scope: Option<String>,
    pub case: DeclaredDeadlineSlot,
    pub hook: DeclaredDeadlineSlot,
    pub fixture: DeclaredDeadlineSlot,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum DeclaredDeadlineSlot {
    Absent,
    Known {
        milliseconds: f64,
        provenance: DeadlineProvenance,
    },
    Unknown {
        reason: DeclaredDeadlineUnknownReason,
        provenance: DeadlineProvenance,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DeclaredDeadlineUnknownReason {
    Expression,
    NonFinite,
    OpaqueSpread,
    OpaqueTestObject,
    ComputedProperty,
    UnresolvedExtends,
    UnresolvedInheritance,
    UnprovedConfigRoot,
    Accessor,
    UnsupportedConfigCall,
    UnprovedBinding,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeadlineProvenance {
    pub path: String,
    /// Half-open OXC byte offsets; None for declarations without source spans.
    pub span: Option<(u32, u32)>,
    pub inherited_through: Vec<DeadlineInheritanceEvidence>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeadlineInheritanceEvidence {
    pub path: String,
    pub project: Option<String>,
}

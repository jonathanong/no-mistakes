//! Data shapes for GitHub Actions artifact upload/download declarations and
//! same-run producer/consumer edges (`actions/upload-artifact`,
//! `actions/download-artifact`).
//!
//! These are pure data shapes ported from a TypeScript engine's
//! `artifact-types.mts`. Field order matches that source's type
//! declarations; see [`super::artifact_values`] and
//! [`super::artifact_resolver`] for the real construction sites these
//! shapes must stay byte-for-byte compatible with, the same way `model.rs`
//! is verified against `parse-workflow.mts`.

use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ArtifactValue {
    #[serde(rename_all = "camelCase")]
    Static {
        raw: String,
        value: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        instance_count: Option<u32>,
    },
    #[serde(rename_all = "camelCase")]
    Finite {
        raw: String,
        values: Vec<String>,
        instance_counts: BTreeMap<String, u32>,
    },
    Dynamic {
        raw: String,
    },
    PathDerived {
        reason: PathDerivedReason,
    },
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum PathDerivedReason {
    ArchiveDisabled,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ArtifactActionFlag {
    Static {
        #[serde(skip_serializing_if = "Option::is_none")]
        raw: Option<String>,
        effective: bool,
    },
    Dynamic {
        raw: String,
    },
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ArtifactUploadDeclaration {
    pub name: ArtifactValue,
    pub archive: ArtifactActionFlag,
    pub overwrite: ArtifactActionFlag,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ArtifactDownloadSelector {
    Name {
        name: ArtifactValue,
    },
    Pattern {
        pattern: ArtifactValue,
    },
    All,
    #[serde(rename_all = "camelCase")]
    ArtifactIds {
        artifact_ids: ArtifactValue,
    },
    #[serde(rename_all = "camelCase")]
    Unresolved {
        reason: UnresolvedReason,
        name: ArtifactValue,
        artifact_ids: ArtifactValue,
    },
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum UnresolvedReason {
    NameWithArtifactIds,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ArtifactDownloadSource {
    #[serde(rename_all = "camelCase")]
    CurrentRun {
        #[serde(skip_serializing_if = "Option::is_none")]
        repository: Option<ArtifactValue>,
        #[serde(skip_serializing_if = "Option::is_none")]
        run_id: Option<ArtifactValue>,
    },
    #[serde(rename_all = "camelCase")]
    External {
        #[serde(skip_serializing_if = "Option::is_none")]
        repository: Option<ArtifactValue>,
        #[serde(skip_serializing_if = "Option::is_none")]
        run_id: Option<ArtifactValue>,
    },
    #[serde(rename_all = "camelCase")]
    Dynamic {
        #[serde(skip_serializing_if = "Option::is_none")]
        repository: Option<ArtifactValue>,
        #[serde(skip_serializing_if = "Option::is_none")]
        run_id: Option<ArtifactValue>,
    },
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ArtifactDownloadDeclaration {
    pub selector: ArtifactDownloadSelector,
    pub source: ArtifactDownloadSource,
}

/// A step's parsed artifact action, when it is an
/// `actions/{upload,download}-artifact` step.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ArtifactDeclaration {
    Upload(ArtifactUploadDeclaration),
    Download(ArtifactDownloadDeclaration),
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ArtifactMatchKind {
    Exact,
    Pattern,
    All,
    Possible,
}

impl ArtifactMatchKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Exact => "exact",
            Self::Pattern => "pattern",
            Self::All => "all",
            Self::Possible => "possible",
        }
    }
}

/// A same-run artifact producer→consumer edge. Wrapped by
/// [`super::model::WorkflowTopologyEdge::Artifact`], which supplies the
/// `kind: "artifact"` tag.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactEdge {
    pub from: String,
    pub to: String,
    pub name: String,
    pub producer_step: u32,
    pub consumer_step: u32,
    #[serde(rename = "match")]
    pub match_kind: ArtifactMatchKind,
}

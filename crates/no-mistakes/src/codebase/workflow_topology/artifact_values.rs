//! Parses `actions/{upload,download}-artifact` step declarations and
//! computes matrix-aware artifact name values, ported from
//! `artifact-values.mts`.
//!
//! `inputs` (the step's raw `with:` mapping) stays a raw `serde_yaml::Value`
//! throughout, matching every other hand-walked step field. `matrix` is the
//! already-[`OrderedJson`]-converted snapshot [`super::workflow_values::matrix_from_job`]
//! also stores on the job — the same conversion, reused, not a second one.

use super::artifact_types::{
    ArtifactActionFlag, ArtifactDeclaration, ArtifactDownloadDeclaration, ArtifactDownloadSelector,
    ArtifactDownloadSource, ArtifactUploadDeclaration, ArtifactValue, PathDerivedReason,
    UnresolvedReason,
};
use super::value_primitives::{self, OrderedJson};
use serde_yaml::Value;

mod matrix_value;

pub use matrix_value::{artifact_value, static_matrix_instance_count};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ArtifactAction {
    Upload,
    Download,
}

/// `uses.match(/^actions\/(upload|download)-artifact@[^/]+$/iu)` —
/// case-insensitive match against `actions/upload-artifact@<ref>` /
/// `actions/download-artifact@<ref>` where `<ref>` has no further `/`.
fn artifact_action(uses: &str) -> Option<ArtifactAction> {
    let lower = uses.to_ascii_lowercase();
    let rest = lower.strip_prefix("actions/")?;
    let (action, ref_part) = if let Some(rest) = rest.strip_prefix("upload-artifact@") {
        (ArtifactAction::Upload, rest)
    } else if let Some(rest) = rest.strip_prefix("download-artifact@") {
        (ArtifactAction::Download, rest)
    } else {
        return None;
    };
    (!ref_part.is_empty() && !ref_part.contains('/')).then_some(action)
}

pub fn parse_artifact_declaration(
    uses: &str,
    inputs: Option<&Value>,
    matrix: Option<&OrderedJson>,
) -> Option<ArtifactDeclaration> {
    let action = artifact_action(uses)?;
    let mapping = inputs.filter(|value| value_primitives::is_record(Some(value)));
    let get = |key: &str| mapping.and_then(|value| value.get(key));

    if action == ArtifactAction::Upload {
        let archive = artifact_flag(get("archive"), true);
        let configured_name =
            value_primitives::string_value(get("name")).unwrap_or_else(|| "artifact".to_string());
        let name = match &archive {
            ArtifactActionFlag::Static {
                effective: true, ..
            } => artifact_value(&configured_name, matrix),
            ArtifactActionFlag::Static {
                effective: false, ..
            } => ArtifactValue::PathDerived {
                reason: PathDerivedReason::ArchiveDisabled,
            },
            ArtifactActionFlag::Dynamic { .. } => ArtifactValue::Dynamic {
                raw: configured_name,
            },
        };
        return Some(ArtifactDeclaration::Upload(ArtifactUploadDeclaration {
            name,
            archive,
            overwrite: artifact_flag(get("overwrite"), false),
        }));
    }

    let name = value_primitives::string_value(get("name"));
    let artifact_ids = value_primitives::string_value(get("artifact-ids"));
    let pattern = value_primitives::string_value(get("pattern"));
    let selector = match (&name, &artifact_ids) {
        (Some(name), Some(artifact_ids)) => ArtifactDownloadSelector::Unresolved {
            reason: UnresolvedReason::NameWithArtifactIds,
            name: artifact_value(name, matrix),
            artifact_ids: artifact_value(artifact_ids, matrix),
        },
        (Some(name), None) => ArtifactDownloadSelector::Name {
            name: artifact_value(name, matrix),
        },
        (None, Some(artifact_ids)) => ArtifactDownloadSelector::ArtifactIds {
            artifact_ids: artifact_value(artifact_ids, matrix),
        },
        (None, None) => match &pattern {
            Some(pattern) => ArtifactDownloadSelector::Pattern {
                pattern: artifact_value(pattern, matrix),
            },
            None => ArtifactDownloadSelector::All,
        },
    };
    Some(ArtifactDeclaration::Download(ArtifactDownloadDeclaration {
        selector,
        source: artifact_download_source(mapping, matrix),
    }))
}

fn artifact_flag(value: Option<&Value>, default_value: bool) -> ArtifactActionFlag {
    let raw = value_primitives::string_value(value);
    if let Some(raw) = &raw {
        if raw.contains("${{") {
            return ArtifactActionFlag::Dynamic { raw: raw.clone() };
        }
    }
    let effective = match &raw {
        Some(raw) => raw.trim().eq_ignore_ascii_case("true"),
        None => default_value,
    };
    ArtifactActionFlag::Static { raw, effective }
}

fn artifact_download_source(
    mapping: Option<&Value>,
    matrix: Option<&OrderedJson>,
) -> ArtifactDownloadSource {
    let get = |key: &str| mapping.and_then(|value| value.get(key));
    let token = value_primitives::string_value(get("github-token"));
    let repository_raw = value_primitives::string_value(get("repository"));
    let run_id_raw = value_primitives::string_value(get("run-id"));
    let repository = repository_raw
        .as_deref()
        .filter(|raw| !raw.is_empty())
        .map(|raw| artifact_value(raw, matrix));
    let run_id = run_id_raw
        .as_deref()
        .filter(|raw| !raw.is_empty())
        .map(|raw| artifact_value(raw, matrix));

    if token.is_none()
        || (is_current_repository(repository_raw.as_deref())
            && is_current_run_id(run_id_raw.as_deref()))
    {
        return ArtifactDownloadSource::CurrentRun { repository, run_id };
    }
    let dynamic = [&repository, &run_id]
        .into_iter()
        .flatten()
        .any(|value| matches!(value, ArtifactValue::Dynamic { .. }));
    if dynamic {
        ArtifactDownloadSource::Dynamic { repository, run_id }
    } else {
        ArtifactDownloadSource::External { repository, run_id }
    }
}

fn is_current_repository(value: Option<&str>) -> bool {
    match value {
        None => true,
        Some(value) => value.trim() == "${{ github.repository }}",
    }
}

fn is_current_run_id(value: Option<&str>) -> bool {
    match value {
        None => true,
        Some(value) => value.trim() == "${{ github.run_id }}",
    }
}

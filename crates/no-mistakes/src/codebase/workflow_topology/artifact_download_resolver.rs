//! Resolves one `actions/download-artifact` step against a run context's
//! upload candidates, ported from `artifact-download-resolver.mts`. This is
//! the core certainty-classification logic: `exact`/`pattern`/`all` when a
//! producer is guaranteed, `possible` when it merely could apply, or a
//! `missing`/`ambiguous` diagnostic when resolution fails outright.

use super::artifact_pattern_match::matches_artifact_pattern;
use super::artifact_resolution_helpers::{
    artifact_edge, artifact_values, can_precede, is_conditional, is_guaranteed_before,
    symbolic_pattern_match,
};
use super::artifact_resolution_types::{
    ArtifactCandidate, ArtifactOccurrence, ArtifactResolution, ArtifactRunContext,
};
use super::artifact_types::{
    ArtifactDownloadDeclaration, ArtifactDownloadSelector, ArtifactDownloadSource, ArtifactEdge,
    ArtifactMatchKind, ArtifactValue,
};
use name_download::resolve_name_download;

mod name_download;

pub fn resolve_artifact_download(
    context: &ArtifactRunContext,
    consumer: &ArtifactOccurrence,
    consumer_step: u32,
    download: &ArtifactDownloadDeclaration,
) -> ArtifactResolution {
    if !matches!(download.source, ArtifactDownloadSource::CurrentRun { .. }) {
        return ArtifactResolution::default();
    }
    match &download.selector {
        ArtifactDownloadSelector::ArtifactIds { .. }
        | ArtifactDownloadSelector::Unresolved { .. } => ArtifactResolution::default(),
        ArtifactDownloadSelector::Name {
            name: ArtifactValue::Dynamic { .. },
        } => ArtifactResolution::default(),
        ArtifactDownloadSelector::Pattern {
            pattern: ArtifactValue::Dynamic { .. },
        } => ArtifactResolution::default(),
        ArtifactDownloadSelector::All => {
            let uploads = artifact_candidates(context, consumer, consumer_step);
            resolve_all_download(context, consumer, consumer_step, &uploads)
        }
        ArtifactDownloadSelector::Pattern { pattern } => {
            let uploads = artifact_candidates(context, consumer, consumer_step);
            resolve_pattern_download(context, consumer, consumer_step, &uploads, pattern)
        }
        ArtifactDownloadSelector::Name { name } => {
            let uploads = artifact_candidates(context, consumer, consumer_step);
            let opaque_possible = context.occurrences.iter().any(|occurrence| {
                occurrence.opaque && can_precede(context, occurrence, 0, consumer, consumer_step)
            });
            resolve_name_download(
                context,
                consumer,
                consumer_step,
                &uploads,
                name,
                opaque_possible,
            )
        }
    }
}

fn artifact_candidates(
    context: &ArtifactRunContext,
    consumer: &ArtifactOccurrence,
    consumer_step: u32,
) -> Vec<ArtifactCandidate> {
    super::artifact_resolution_helpers::artifact_candidates(context, consumer, consumer_step)
}

fn resolve_all_download(
    context: &ArtifactRunContext,
    consumer: &ArtifactOccurrence,
    consumer_step: u32,
    uploads: &[ArtifactCandidate],
) -> ArtifactResolution {
    let edges = uploads
        .iter()
        .map(|candidate| {
            let match_kind = possible_or(
                is_conditional(candidate)
                    || !is_guaranteed_before(
                        context,
                        &candidate.occurrence,
                        candidate.step_index,
                        consumer,
                        consumer_step,
                    ),
                ArtifactMatchKind::All,
            );
            artifact_edge(candidate, consumer, consumer_step, match_kind, None)
        })
        .collect();
    ArtifactResolution {
        edges,
        diagnostic: None,
    }
}

fn resolve_pattern_download(
    context: &ArtifactRunContext,
    consumer: &ArtifactOccurrence,
    consumer_step: u32,
    uploads: &[ArtifactCandidate],
    pattern: &ArtifactValue,
) -> ArtifactResolution {
    let patterns = artifact_values(pattern);
    let edges = uploads
        .iter()
        .flat_map(|candidate| pattern_edges(context, candidate, &patterns, consumer, consumer_step))
        .collect();
    ArtifactResolution {
        edges,
        diagnostic: None,
    }
}

fn pattern_edges(
    context: &ArtifactRunContext,
    candidate: &ArtifactCandidate,
    patterns: &[String],
    consumer: &ArtifactOccurrence,
    consumer_step: u32,
) -> Vec<ArtifactEdge> {
    let upload_name = &candidate.upload.name;
    let concrete: Vec<String> = artifact_values(upload_name)
        .into_iter()
        .filter(|name| {
            patterns
                .iter()
                .any(|pattern| matches_artifact_pattern(name, pattern))
        })
        .collect();
    if !concrete.is_empty() {
        let match_kind = possible_or(
            is_conditional(candidate)
                || !is_guaranteed_before(
                    context,
                    &candidate.occurrence,
                    candidate.step_index,
                    consumer,
                    consumer_step,
                ),
            ArtifactMatchKind::Pattern,
        );
        return concrete
            .into_iter()
            .map(|name| artifact_edge(candidate, consumer, consumer_step, match_kind, Some(&name)))
            .collect();
    }
    if let ArtifactValue::Dynamic { raw } = upload_name {
        if patterns
            .iter()
            .any(|pattern| symbolic_pattern_match(raw, pattern))
        {
            return vec![artifact_edge(
                candidate,
                consumer,
                consumer_step,
                ArtifactMatchKind::Possible,
                None,
            )];
        }
    }
    Vec::new()
}

fn possible_or(uncertain: bool, certain: ArtifactMatchKind) -> ArtifactMatchKind {
    if uncertain {
        ArtifactMatchKind::Possible
    } else {
        certain
    }
}

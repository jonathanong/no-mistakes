//! The exact-`name` download resolution path — missing/ambiguous producer
//! detection and `exact`-vs-`possible` certainty classification — split out
//! of [`super`] to stay under the crate's per-file line limit.

use super::super::artifact_resolution_diagnostics::{
    ambiguous_artifact_producer, missing_artifact_producer, AmbiguousGroup,
};
use super::super::artifact_resolution_helpers::{
    artifact_edge, artifact_values, candidate_before, candidate_instance_count, is_conditional,
    is_guaranteed_before, supersedes,
};
use super::super::artifact_resolution_types::{
    ArtifactCandidate, ArtifactOccurrence, ArtifactResolution, ArtifactRunContext,
};
use super::super::artifact_types::{ArtifactMatchKind, ArtifactValue};
use super::possible_or;

pub(super) fn resolve_name_download(
    context: &ArtifactRunContext,
    consumer: &ArtifactOccurrence,
    consumer_step: u32,
    uploads: &[ArtifactCandidate],
    requested: &ArtifactValue,
    opaque_possible: bool,
) -> ArtifactResolution {
    let requested_names = artifact_values(requested);
    let dynamic_possible = uploads
        .iter()
        .any(|candidate| matches!(candidate.upload.name, ArtifactValue::Dynamic { .. }));
    let path_derived_possible = uploads
        .iter()
        .any(|candidate| matches!(candidate.upload.name, ArtifactValue::PathDerived { .. }));
    let unresolved_possible = dynamic_possible || path_derived_possible || opaque_possible;

    let candidates_by_name: Vec<(String, Vec<&ArtifactCandidate>)> = requested_names
        .into_iter()
        .map(|name| {
            let candidates: Vec<&ArtifactCandidate> = uploads
                .iter()
                .filter(|candidate| artifact_values(&candidate.upload.name).contains(&name))
                .collect();
            (name, candidates)
        })
        .collect();

    let missing_names: Vec<String> = candidates_by_name
        .iter()
        .filter(|(_, candidates)| candidates.is_empty())
        .map(|(name, _)| name.clone())
        .collect();
    if !missing_names.is_empty() && !unresolved_possible {
        return missing_artifact_producer(consumer, consumer_step, &missing_names);
    }

    let effective_by_name: Vec<(String, Vec<&ArtifactCandidate>)> = candidates_by_name
        .into_iter()
        .filter(|(_, candidates)| !candidates.is_empty())
        .map(|(name, candidates)| {
            let effective = candidates
                .iter()
                .copied()
                .filter(|candidate| {
                    !candidates
                        .iter()
                        .any(|later| supersedes(context, candidate, later, consumer, consumer_step))
                })
                .collect();
            (name, effective)
        })
        .collect();

    let ambiguous_groups: Vec<&(String, Vec<&ArtifactCandidate>)> = effective_by_name
        .iter()
        .filter(|(name, candidates)| {
            candidates.iter().enumerate().any(|(index, left)| {
                (!is_conditional(left) && candidate_instance_count(left, name).unwrap_or(1) > 1)
                    || candidates[index + 1..].iter().any(|right| {
                        !is_conditional(left)
                            && !is_conditional(right)
                            && !candidate_before(context, left, right)
                            && !candidate_before(context, right, left)
                    })
            })
        })
        .collect();
    if !ambiguous_groups.is_empty() {
        let groups: Vec<AmbiguousGroup<'_>> = ambiguous_groups
            .iter()
            .map(|(name, candidates)| AmbiguousGroup {
                name,
                candidates: candidates.as_slice(),
            })
            .collect();
        return ambiguous_artifact_producer(consumer, consumer_step, &groups);
    }

    let edges = effective_by_name
        .iter()
        .flat_map(|(name, candidates)| {
            candidates.iter().map(move |candidate| {
                let match_kind = possible_or(
                    candidates.len() > 1
                        || unresolved_possible
                        || is_conditional(candidate)
                        || !is_guaranteed_before(
                            context,
                            &candidate.occurrence,
                            candidate.step_index,
                            consumer,
                            consumer_step,
                        )
                        || candidate_instance_count(candidate, name).is_none(),
                    ArtifactMatchKind::Exact,
                );
                artifact_edge(candidate, consumer, consumer_step, match_kind, Some(name))
            })
        })
        .collect();
    ArtifactResolution {
        edges,
        diagnostic: None,
    }
}

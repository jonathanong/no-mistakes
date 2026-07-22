//! Shared graph-topology and value-shape helpers for same-run artifact
//! resolution, ported from `artifact-resolution-helpers.mts`.

use super::artifact_resolution_types::{ArtifactCandidate, ArtifactOccurrence, ArtifactRunContext};
use super::artifact_types::{ArtifactActionFlag, ArtifactEdge, ArtifactMatchKind, ArtifactValue};

mod edge_keys;
mod symbolic_pattern;

pub use edge_keys::{diagnostic_key, edge_key, edge_set_key, occurrence_reaches, unique_edges};
pub use symbolic_pattern::symbolic_pattern_match;

/// Whether `producer`'s step could run before `consumer`'s: the same
/// occurrence compares step indexes directly; different occurrences compare
/// via `needs`-derived reachability (a producer can precede a consumer
/// unless the consumer is already known to reach the producer, which would
/// make the producer strictly *after* it).
pub fn can_precede(
    context: &ArtifactRunContext,
    producer: &ArtifactOccurrence,
    producer_step: u32,
    consumer: &ArtifactOccurrence,
    consumer_step: u32,
) -> bool {
    if producer.id == consumer.id {
        return producer_step < consumer_step;
    }
    !occurrence_reaches(context, &consumer.id, &producer.id)
}

/// Every upload step, across every occurrence in this run context, whose
/// step could precede `consumer`'s download step.
pub fn artifact_candidates(
    context: &ArtifactRunContext,
    consumer: &ArtifactOccurrence,
    consumer_step: u32,
) -> Vec<ArtifactCandidate> {
    context
        .occurrences
        .iter()
        .flat_map(|occurrence| {
            occurrence.job.steps.iter().filter_map(move |step| {
                let Some(super::artifact_types::ArtifactDeclaration::Upload(upload)) =
                    &step.artifact
                else {
                    return None;
                };
                can_precede(context, occurrence, step.index, consumer, consumer_step).then(|| {
                    ArtifactCandidate {
                        occurrence: occurrence.clone(),
                        step_index: step.index,
                        upload: upload.clone(),
                    }
                })
            })
        })
        .collect()
}

/// Whether `producer`'s step is *guaranteed* to run before `consumer`'s
/// (the same occurrence with an earlier index, or genuine `needs`
/// reachability) — stronger than [`can_precede`], which only rules out the
/// reverse.
pub fn is_guaranteed_before(
    context: &ArtifactRunContext,
    producer: &ArtifactOccurrence,
    producer_step: u32,
    consumer: &ArtifactOccurrence,
    consumer_step: u32,
) -> bool {
    if producer.id == consumer.id {
        producer_step < consumer_step
    } else {
        occurrence_reaches(context, &producer.id, &consumer.id)
    }
}

/// Whether `later`'s upload is a guaranteed, unconditional, `overwrite:
/// true` re-upload of the same name that strictly follows `earlier`'s and
/// is itself guaranteed before the consumer — meaning `earlier` can never
/// be the artifact the consumer actually sees.
pub fn supersedes(
    context: &ArtifactRunContext,
    earlier: &ArtifactCandidate,
    later: &ArtifactCandidate,
    consumer: &ArtifactOccurrence,
    consumer_step: u32,
) -> bool {
    candidate_before(context, earlier, later)
        && is_guaranteed_before(
            context,
            &later.occurrence,
            later.step_index,
            consumer,
            consumer_step,
        )
        && matches!(
            &later.upload.overwrite,
            ArtifactActionFlag::Static {
                effective: true,
                ..
            }
        )
        && !is_conditional(later)
}

pub fn candidate_before(
    context: &ArtifactRunContext,
    earlier: &ArtifactCandidate,
    later: &ArtifactCandidate,
) -> bool {
    if earlier.occurrence.id == later.occurrence.id {
        earlier.step_index < later.step_index
    } else {
        occurrence_reaches(context, &earlier.occurrence.id, &later.occurrence.id)
    }
}

/// Whether `candidate`'s upload runs conditionally — either because it was
/// reached through a conditional job (its own `if:` or an ancestor's, for a
/// job reached via a local reusable-workflow call), or the step itself has
/// an `if:`. A conditional producer can never be a *guaranteed* one.
pub fn is_conditional(candidate: &ArtifactCandidate) -> bool {
    candidate.occurrence.inherited_conditional
        || candidate
            .occurrence
            .job
            .steps
            .get(candidate.step_index as usize)
            .is_some_and(|step| step.condition.is_some())
}

pub fn artifact_edge(
    candidate: &ArtifactCandidate,
    consumer: &ArtifactOccurrence,
    consumer_step: u32,
    match_kind: ArtifactMatchKind,
    name: Option<&str>,
) -> ArtifactEdge {
    let name = name
        .map(str::to_string)
        .unwrap_or_else(|| artifact_value_label(&candidate.upload.name));
    ArtifactEdge {
        from: candidate.occurrence.job.id.clone(),
        to: consumer.job.id.clone(),
        name,
        producer_step: candidate.step_index,
        consumer_step,
        match_kind,
    }
}

pub fn artifact_values(value: &ArtifactValue) -> Vec<String> {
    match value {
        ArtifactValue::Static { value, .. } => vec![value.clone()],
        ArtifactValue::Finite { values, .. } => values.clone(),
        ArtifactValue::Dynamic { .. } | ArtifactValue::PathDerived { .. } => Vec::new(),
    }
}

pub fn artifact_instance_count(value: &ArtifactValue, name: &str) -> Option<u32> {
    match value {
        ArtifactValue::Static {
            value,
            instance_count,
            ..
        } if value == name => Some(instance_count.unwrap_or(1)),
        ArtifactValue::Finite {
            instance_counts, ..
        } => instance_counts.get(name).copied(),
        _ => None,
    }
}

pub fn candidate_instance_count(candidate: &ArtifactCandidate, name: &str) -> Option<u32> {
    let upload_count = artifact_instance_count(&candidate.upload.name, name)?;
    let invocation_count = candidate.occurrence.invocation_count?;
    Some(upload_count * invocation_count)
}

fn artifact_value_label(value: &ArtifactValue) -> String {
    match value {
        ArtifactValue::PathDerived { .. } => "<path-derived: archive-disabled>".to_string(),
        ArtifactValue::Static { raw, .. }
        | ArtifactValue::Finite { raw, .. }
        | ArtifactValue::Dynamic { raw } => raw.clone(),
    }
}

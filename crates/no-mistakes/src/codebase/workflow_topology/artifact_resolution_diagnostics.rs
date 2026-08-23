//! Builds the two same-run artifact-resolution error diagnostics, ported
//! from `artifact-resolution-diagnostics.mts`.

use super::artifact_resolution_helpers::candidate_instance_count;
use super::artifact_resolution_types::{ArtifactCandidate, ArtifactOccurrence, ArtifactResolution};
use super::model::{DiagnosticCode, WorkflowTopologyDiagnostic};

pub fn missing_artifact_producer(
    consumer: &ArtifactOccurrence,
    consumer_step: u32,
    missing_names: &[String],
) -> ArtifactResolution {
    ArtifactResolution {
        edges: Vec::new(),
        diagnostic: Some(
            WorkflowTopologyDiagnostic::new(
                DiagnosticCode::MissingArtifactProducer,
                format!(
                    "{} step {consumer_step} downloads {} but no same-run producer can precede it",
                    consumer.job.id,
                    missing_names.join(", "),
                ),
                consumer.job.workflow_id.clone(),
            )
            .with_job(consumer.job.id.clone())
            .with_step(consumer_step),
        ),
    }
}

/// One name's competing candidates, for the diagnostic message built by
/// [`ambiguous_artifact_producer`].
pub struct AmbiguousGroup<'a> {
    pub name: &'a str,
    pub candidates: &'a [&'a ArtifactCandidate],
}

pub fn ambiguous_artifact_producer(
    consumer: &ArtifactOccurrence,
    consumer_step: u32,
    groups: &[AmbiguousGroup<'_>],
) -> ArtifactResolution {
    let mut descriptions: Vec<String> = groups
        .iter()
        .flat_map(|group| {
            group.candidates.iter().map(move |candidate| {
                let count = candidate_instance_count(candidate, group.name);
                let multiplicity = match count {
                    Some(count) if count > 1 => format!(" ({count} matrix instances)"),
                    _ => String::new(),
                };
                format!(
                    "{}: {} step {}{multiplicity}",
                    group.name, candidate.occurrence.job.id, candidate.step_index
                )
            })
        })
        .collect();
    descriptions.sort();
    ArtifactResolution {
        edges: Vec::new(),
        diagnostic: Some(
            WorkflowTopologyDiagnostic::new(
                DiagnosticCode::AmbiguousArtifactProducer,
                format!(
                    "{} step {consumer_step} has competing artifact producers: {}",
                    consumer.job.id,
                    descriptions.join("; "),
                ),
                consumer.job.workflow_id.clone(),
            )
            .with_job(consumer.job.id.clone())
            .with_step(consumer_step),
        ),
    }
}

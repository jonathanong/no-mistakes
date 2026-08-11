use serde_yaml::Value;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use super::model::ReusableTarget;
use crate::codebase::workflow_topology::model::WorkflowCallEdge;

mod contracts;
mod jobs;
mod matrix;
mod workflow;

pub(super) use contracts::workflow_call_shape_valid;
pub(super) use jobs::{
    call_bindings_shape_valid, container_images_valid_for_inputs, reusable_call_job_shape_valid,
    steps_shape_valid,
};
pub(super) use matrix::{static_matrix_combinations, zero_instance_matrix, MatrixCombinations};
pub(super) use workflow::workflow_shape_valid;

pub(super) fn scan_job_shape_valid(job: &Value) -> bool {
    matrix::matrix_shape_valid(job)
        && steps_shape_valid(job)
        && if job.get("uses").is_some() {
            reusable_call_job_shape_valid(job)
        } else {
            jobs::step_job_shape_valid(job)
        }
}

pub(super) fn canonical_local_call_target(target: &str) -> bool {
    target
        .strip_prefix("./.github/workflows/")
        .is_some_and(|filename| {
            !filename.is_empty()
                && !filename.contains(['/', '\\'])
                && (filename.ends_with(".yml") || filename.ends_with(".yaml"))
        })
}

pub(super) fn canonical_remote_call_target(target: &str) -> bool {
    let Some((path, reference)) = target.rsplit_once('@') else {
        return false;
    };
    if !valid_remote_reference(reference) || path.contains('@') {
        return false;
    }
    let mut segments = path.split('/');
    let valid = matches!(
        (
            segments.next(),
            segments.next(),
            segments.next(),
            segments.next(),
            segments.next(),
        ),
        (Some(owner), Some(repository), Some(".github"), Some("workflows"), Some(filename))
            if valid_remote_owner(owner)
                && valid_remote_repository(repository)
                && canonical_workflow_filename(filename)
    );
    valid && segments.next().is_none()
}

pub(super) fn valid_remote_owner(owner: &str) -> bool {
    !owner.is_empty()
        && owner.len() <= 39
        && !owner.starts_with('-')
        && !owner.ends_with('-')
        && !owner.contains("--")
        && owner
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
}

pub(super) fn valid_remote_repository(repository: &str) -> bool {
    !repository.is_empty()
        && repository.len() <= 100
        && !matches!(repository, "." | "..")
        && repository
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

pub(super) fn valid_remote_reference(reference: &str) -> bool {
    !reference.is_empty()
        && reference != "@"
        && !reference.contains("${{")
        && !reference.starts_with(['/', '.'])
        && !reference.ends_with(['/', '.'])
        && !reference.contains([' ', '~', '^', ':', '?', '*', '[', '\\'])
        && !reference.contains("..")
        && !reference.contains("@{")
        && !reference.contains("//")
        && !reference
            .split('/')
            .any(|component| component.ends_with(".lock"))
        && reference.bytes().all(|byte| byte >= 0x20 && byte != 0x7f)
}

pub(super) fn validated_reusable_target(edge: &WorkflowCallEdge) -> Option<ReusableTarget> {
    if edge.local {
        let path = edge.to.clone()?;
        canonical_local_call_target(&edge.target).then_some(ReusableTarget::Local(path))
    } else {
        canonical_remote_call_target(&edge.target)
            .then(|| ReusableTarget::Remote(edge.target.clone()))
    }
}

fn canonical_workflow_filename(filename: &str) -> bool {
    !filename.is_empty()
        && !filename.contains(['/', '\\'])
        && (filename.ends_with(".yml") || filename.ends_with(".yaml"))
}

pub(super) fn valid_job_dependencies(jobs: &serde_yaml::Mapping) -> bool {
    let mut dependencies = BTreeMap::new();
    for (job_id, job) in jobs {
        let Some(job_id) = super::super::normalized_job_id(job_id) else {
            return false;
        };
        if !valid_job_id(&job_id) {
            return false;
        }
        let Some(job) = job.as_mapping() else {
            return false;
        };
        if job.get("needs").is_some_and(|needs| {
            !matches!(needs, Value::String(_))
                && !needs
                    .as_sequence()
                    .is_some_and(|items| items.iter().all(Value::is_string))
        }) {
            return false;
        }
        let needs =
            crate::codebase::workflow_topology::value_primitives::string_list(job.get("needs"))
                .into_iter()
                .map(|need| need.to_lowercase())
                .collect::<BTreeSet<_>>();
        if dependencies.insert(job_id, needs).is_some() {
            return false;
        }
    }
    if dependencies
        .values()
        .flatten()
        .any(|need| !dependencies.contains_key(need))
    {
        return false;
    }
    let mut dependents = dependencies
        .keys()
        .map(|job_id| (job_id.clone(), Vec::new()))
        .collect::<BTreeMap<_, _>>();
    let mut remaining_needs = BTreeMap::new();
    for (job_id, needs) in &dependencies {
        remaining_needs.insert(job_id.clone(), needs.len());
        for need in needs {
            dependents
                .get_mut(need)
                .expect("dependency existence was validated")
                .push(job_id.clone());
        }
    }
    let mut ready = remaining_needs
        .iter()
        .filter_map(|(job_id, count)| (*count == 0).then_some(job_id.clone()))
        .collect::<VecDeque<_>>();
    let mut visited = 0;
    while let Some(job_id) = ready.pop_front() {
        visited += 1;
        for dependent in &dependents[&job_id] {
            let count = remaining_needs
                .get_mut(dependent)
                .expect("dependent was collected from the same job graph");
            *count -= 1;
            if *count == 0 {
                ready.push_back(dependent.clone());
            }
        }
    }
    visited == dependencies.len()
}

fn valid_job_id(job_id: &str) -> bool {
    let mut characters = job_id.chars();
    characters
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic() || first == '_')
        && characters
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '-'))
}

#[cfg(test)]
mod tests;

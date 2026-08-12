use super::{comments::strip_static_comments, static_tokens};

mod outcomes;
use outcomes::{static_group_outcome, GroupOutcome};

pub(crate) fn shell_body_has_static_failure(script: &str) -> bool {
    shell_body_has_static_failure_with_initial(script, true)
}

pub(crate) fn shell_body_has_static_failure_with_initial(
    script: &str,
    failure_enforced: bool,
) -> bool {
    let script = strip_static_comments(script);
    let groups = static_groups(&script);
    let mut previous_failed = None;
    for (index, group) in groups.iter().enumerate() {
        let outcome = static_group_outcome(group, previous_failed);
        previous_failed = outcome.failed();
        match outcome {
            GroupOutcome::Exit(failed) => return failed,
            GroupOutcome::Return if failure_enforced => return true,
            GroupOutcome::Return if index + 1 == groups.len() => return true,
            GroupOutcome::Failure { errexit: true } if failure_enforced => return true,
            GroupOutcome::Failure { .. } if index + 1 == groups.len() => return true,
            GroupOutcome::Success
            | GroupOutcome::Return
            | GroupOutcome::Failure { .. }
            | GroupOutcome::Unknown => {}
        }
    }
    false
}

pub(crate) fn shell_body_is_statically_successful(script: &str) -> bool {
    let script = strip_static_comments(script);
    let groups = static_groups(&script);
    if groups.is_empty() {
        return false;
    }
    let mut previous_failed = None;
    for (index, group) in groups.iter().enumerate() {
        let outcome = static_group_outcome(group, previous_failed);
        previous_failed = outcome.failed();
        match outcome {
            GroupOutcome::Success => {}
            GroupOutcome::Exit(false) => return true,
            // A non-terminal AND-list failure can be followed by a known
            // successful command. Preserve it for a later bare `exit`, but
            // let that later command establish the final body status.
            GroupOutcome::Failure { .. } if index + 1 < groups.len() => {}
            GroupOutcome::Failure { .. } => return false,
            GroupOutcome::Return | GroupOutcome::Exit(true) | GroupOutcome::Unknown => {
                return false
            }
        }
    }
    true
}

pub(crate) fn shell_body_has_static_pipeline_failure(script: &str, failure_enforced: bool) -> bool {
    let script = strip_static_comments(script);
    let groups = static_groups(&script);
    groups.iter().enumerate().any(|(index, group)| {
        terminating_pipeline_segment(group, failure_enforced, index + 1 == groups.len()).is_some()
    })
}

/// Returns the statically reachable prefix before the first pipefail pipeline
/// that terminates the shell body. Commands before that pipeline ran; commands
/// after it did not.
pub(crate) fn shell_body_before_static_pipeline_failure(
    script: &str,
    failure_enforced: bool,
) -> String {
    let script = strip_static_comments(script);
    let groups = static_groups(&script);
    let mut reachable_groups = Vec::new();
    for (index, group) in groups.iter().enumerate() {
        let Some(segment_index) =
            terminating_pipeline_segment(group, failure_enforced, index + 1 == groups.len())
        else {
            reachable_groups.push((*group).to_string());
            continue;
        };
        let prefix = group
            .split("&&")
            .take(segment_index)
            .map(str::trim)
            .collect::<Vec<_>>()
            .join("&&");
        if !prefix.trim().is_empty() {
            reachable_groups.push(prefix.trim().to_string());
        }
        return reachable_groups.join("\n");
    }
    script
}

pub(crate) fn shell_body_before_static_failure(
    script: &str,
    failure_enforced: bool,
    pipefail_enforced: bool,
) -> String {
    let script = if pipefail_enforced {
        shell_body_before_static_pipeline_failure(script, failure_enforced)
    } else {
        strip_static_comments(script)
    };
    let groups = static_groups(&script);
    let mut previous_failed = None;
    for (index, group) in groups.iter().enumerate() {
        let outcome = static_group_outcome(group, previous_failed);
        previous_failed = outcome.failed();
        if matches!(outcome, GroupOutcome::Exit(_))
            || failure_enforced && matches!(outcome, GroupOutcome::Return)
            || failure_enforced && matches!(outcome, GroupOutcome::Failure { errexit: true })
        {
            return groups[..index].join("\n");
        }
    }
    script
}

fn static_groups(script: &str) -> Vec<&str> {
    script
        .split(['\n', ';'])
        .filter(|group| !group.trim().is_empty())
        .collect()
}

fn terminating_pipeline_segment(
    group: &str,
    failure_enforced: bool,
    final_group: bool,
) -> Option<usize> {
    if group.contains("||") {
        return None;
    }
    let segments = group.split("&&").collect::<Vec<_>>();
    for (index, segment) in segments.iter().enumerate() {
        if segment.split('|').count() > 1 {
            match static_pipeline_failure(segment) {
                Some(true) if final_group || (index + 1 == segments.len() && failure_enforced) => {
                    return Some(index);
                }
                Some(false) => continue,
                Some(true) | None => return None,
            }
        }
        match static_group_outcome(segment, None) {
            GroupOutcome::Success => {}
            // In a final group, either a dynamic predecessor or the failing
            // pipeline produces a failing final status. Earlier groups can
            // continue after a failed AND-list predecessor, so require a
            // proven path to the pipeline before truncating there.
            GroupOutcome::Unknown if final_group => {}
            GroupOutcome::Failure { .. }
            | GroupOutcome::Return
            | GroupOutcome::Exit(_)
            | GroupOutcome::Unknown => {
                return None;
            }
        }
    }
    None
}

fn static_pipeline_failure(segment: &str) -> Option<bool> {
    let mut failed = false;
    let mut unknown = false;
    for command in segment.split('|') {
        match static_group_outcome(command, None) {
            GroupOutcome::Success | GroupOutcome::Exit(false) => {}
            GroupOutcome::Failure { .. } | GroupOutcome::Return | GroupOutcome::Exit(true) => {
                failed = true
            }
            GroupOutcome::Unknown => unknown = true,
        }
    }
    if failed {
        Some(true)
    } else if unknown {
        None
    } else {
        Some(false)
    }
}

#[cfg(test)]
mod tests;

use super::{comments::strip_static_comments, static_tokens};

pub(crate) fn shell_body_has_static_failure(script: &str) -> bool {
    let script = strip_static_comments(script);
    let groups = script.split(['\n', ';']).collect::<Vec<_>>();
    for (index, group) in groups.iter().enumerate() {
        match static_group_outcome(group) {
            GroupOutcome::Exit(failed) => return failed,
            GroupOutcome::Failure { errexit: true } => return true,
            GroupOutcome::Failure { errexit: false } if index + 1 == groups.len() => return true,
            GroupOutcome::Success
            | GroupOutcome::Failure { errexit: false }
            | GroupOutcome::Unknown => {}
        }
    }
    false
}

pub(crate) fn shell_body_has_static_terminal_failure(script: &str) -> bool {
    let script = strip_static_comments(script);
    let groups = script.split(['\n', ';']).collect::<Vec<_>>();
    for (index, group) in groups.iter().enumerate() {
        match static_group_outcome(group) {
            GroupOutcome::Exit(failed) => return failed,
            GroupOutcome::Failure { .. } if index + 1 == groups.len() => return true,
            GroupOutcome::Success | GroupOutcome::Failure { .. } | GroupOutcome::Unknown => {}
        }
    }
    false
}

pub(crate) fn shell_body_has_static_pipeline_failure(script: &str, failure_enforced: bool) -> bool {
    let script = strip_static_comments(script);
    let groups = script.split(['\n', ';']).collect::<Vec<_>>();
    groups.iter().enumerate().any(|(index, group)| {
        if group.contains("||") {
            return false;
        }
        let pipeline_fails = reachable_and_list_pipeline_fails(group);
        pipeline_fails && ((!group.contains("&&") && failure_enforced) || index + 1 == groups.len())
    })
}

fn reachable_and_list_pipeline_fails(group: &str) -> bool {
    for segment in group.split("&&") {
        if segment.split('|').count() > 1 {
            match static_pipeline_failure(segment) {
                Some(true) => return true,
                Some(false) => continue,
                None => return false,
            }
        }
        if !matches!(static_group_outcome(segment), GroupOutcome::Success) {
            return false;
        }
    }
    false
}

fn static_pipeline_failure(segment: &str) -> Option<bool> {
    let mut failed = false;
    for command in segment.split('|') {
        match static_group_outcome(command) {
            GroupOutcome::Success | GroupOutcome::Exit(false) => {}
            GroupOutcome::Failure { .. } | GroupOutcome::Exit(true) => failed = true,
            GroupOutcome::Unknown => return None,
        }
    }
    Some(failed)
}

enum GroupOutcome {
    Success,
    Failure { errexit: bool },
    Exit(bool),
    Unknown,
}

fn static_group_outcome(group: &str) -> GroupOutcome {
    let commands = group.split("&&").collect::<Vec<_>>();
    for (index, command) in commands.iter().enumerate() {
        let Some(tokens) = static_tokens(command) else {
            return GroupOutcome::Unknown;
        };
        if tokens.iter().any(|token| token == "||") {
            return GroupOutcome::Unknown;
        }
        match tokens.first().map(String::as_str) {
            Some("true") => {}
            Some("false") => {
                return GroupOutcome::Failure {
                    errexit: index + 1 == commands.len(),
                };
            }
            Some("return") => return GroupOutcome::Exit(true),
            Some("exit") => return GroupOutcome::Exit(exit_status_fails(&tokens)),
            _ => return GroupOutcome::Unknown,
        }
    }
    GroupOutcome::Success
}

fn exit_status_fails(tokens: &[String]) -> bool {
    match tokens {
        [command, status] if command == "exit" => status
            .parse::<i64>()
            .map_or(true, |status| status.rem_euclid(256) != 0),
        [command] if command == "exit" => false,
        _ => true,
    }
}

#[cfg(test)]
mod tests;

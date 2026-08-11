use super::static_tokens;

#[derive(Clone, Copy)]
pub(super) enum GroupOutcome {
    Success,
    Failure { errexit: bool },
    Exit(bool),
    Unknown,
}

impl GroupOutcome {
    pub(super) fn failed(self) -> Option<bool> {
        match self {
            Self::Success | Self::Exit(false) => Some(false),
            Self::Failure { .. } | Self::Exit(true) => Some(true),
            Self::Unknown => None,
        }
    }
}

pub(super) fn static_group_outcome(group: &str, mut previous_failed: Option<bool>) -> GroupOutcome {
    let commands = group.split("&&").collect::<Vec<_>>();
    for (index, command) in commands.iter().enumerate() {
        if command.contains("||") {
            return GroupOutcome::Unknown;
        }
        let Some(tokens) = static_tokens(command) else {
            return GroupOutcome::Unknown;
        };
        match tokens.first().map(String::as_str) {
            Some("true") => previous_failed = Some(false),
            Some("false") => {
                return GroupOutcome::Failure {
                    errexit: index + 1 == commands.len(),
                };
            }
            Some("return") => return GroupOutcome::Exit(true),
            Some("exit") => {
                return exit_status_fails(&tokens, previous_failed)
                    .map_or(GroupOutcome::Unknown, GroupOutcome::Exit)
            }
            _ => return GroupOutcome::Unknown,
        }
    }
    GroupOutcome::Success
}

fn exit_status_fails(tokens: &[String], previous_failed: Option<bool>) -> Option<bool> {
    match tokens {
        [command, status] if command == "exit" => Some(
            status
                .parse::<i64>()
                .map_or(true, |status| status.rem_euclid(256) != 0),
        ),
        [command] if command == "exit" => previous_failed,
        _ => Some(true),
    }
}

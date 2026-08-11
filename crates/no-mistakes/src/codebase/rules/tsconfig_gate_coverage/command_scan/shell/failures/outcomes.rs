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
        let Some(tokens) = outcome_tokens(&tokens) else {
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
                return exit_status_fails(tokens, previous_failed)
                    .map_or(GroupOutcome::Unknown, GroupOutcome::Exit)
            }
            _ => return GroupOutcome::Unknown,
        }
    }
    GroupOutcome::Success
}

fn outcome_tokens(mut tokens: &[String]) -> Option<&[String]> {
    loop {
        match tokens.first()?.as_str() {
            "builtin" => {
                tokens = match tokens.get(1)?.as_str() {
                    "--" => tokens.get(2..).filter(|tokens| !tokens.is_empty())?,
                    argument if argument.starts_with('-') => return None,
                    _ => tokens.get(1..)?,
                };
            }
            "command" => {
                let mut index = 1;
                while tokens.get(index).is_some_and(|argument| {
                    argument.strip_prefix('-').is_some_and(|flags| {
                        !flags.is_empty() && flags.chars().all(|flag| flag == 'p')
                    })
                }) {
                    index += 1;
                }
                if tokens.get(index).map(String::as_str) == Some("--") {
                    index += 1;
                }
                if tokens
                    .get(index)
                    .is_some_and(|argument| argument.starts_with('-'))
                {
                    return None;
                }
                tokens = tokens.get(index..).filter(|tokens| !tokens.is_empty())?;
            }
            _ => return Some(tokens),
        }
    }
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

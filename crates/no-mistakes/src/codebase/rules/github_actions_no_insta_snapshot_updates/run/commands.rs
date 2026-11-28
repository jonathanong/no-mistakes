use crate::codebase::rules::github_actions_download_integrity::words::{self, Word};

mod cargo;
use cargo::{cargo, insta};

const MAX_NESTING: usize = 4;

pub(super) fn mentions_insta(script: &str) -> bool {
    let bytes = script.as_bytes();
    let identifier = |byte: &u8| byte.is_ascii_alphanumeric() || *byte == b'_';
    bytes.windows(5).enumerate().any(|(start, word)| {
        word.eq_ignore_ascii_case(b"insta")
            && (start == 0 || !identifier(&bytes[start - 1]))
            && !bytes.get(start + 5).is_some_and(identifier)
    })
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Check {
    Accept(&'static str),
    Unknown(&'static str),
}

pub(super) fn checks(script: &str) -> Vec<Check> {
    let mut checks = Vec::new();
    inspect_script(script, 0, &mut checks);
    checks.sort_unstable();
    checks.dedup();
    checks
}

fn inspect_script(script: &str, depth: usize, checks: &mut Vec<Check>) {
    let parsed = words::parse_ordered(script, true, false);
    let mentions_insta = mentions_insta(script);
    let opaque = if parsed.unterminated_heredoc
        || parsed.unterminated_quote
        || parsed.unterminated_substitution
    {
        Some("contains incomplete shell syntax")
    } else if depth > MAX_NESTING {
        Some("contains shell scripts nested too deeply to check")
    } else {
        None
    };
    if let Some(reason) = opaque {
        if mentions_insta || parsed.commands.iter().any(dynamic_invocation) {
            checks.push(Check::Unknown(reason));
        }
        return;
    }
    for command in parsed.commands {
        let Some((name, index)) = words::command_at(&command.words) else {
            continue;
        };
        let command_word = &command.words[index];
        let args = &command.words[index + 1..];
        let result = if command_word.text.contains("${{") {
            Some(Check::Unknown("uses a GitHub expression in `run`"))
        } else if dynamic(command_word, command.dynamic) {
            Some(Check::Unknown("invokes a dynamic command"))
        } else {
            match name.as_str() {
                "cargo" => cargo(args, command.dynamic),
                "cargo-insta" => insta(args, command.dynamic),
                "eval" | "source" | "."
                    if mentions_insta || args.iter().any(|word| dynamic(word, command.dynamic)) =>
                {
                    Some(Check::Unknown("invokes a dynamic command"))
                }
                "case" | "for" | "select" if mentions_insta => {
                    Some(Check::Unknown("invokes a dynamic command"))
                }
                shell if is_shell(shell) => match args {
                    [flag, script, ..]
                        if flag.text == "-c" && !dynamic(script, command.dynamic) =>
                    {
                        inspect_script(&script.text, depth + 1, checks);
                        None
                    }
                    _ if mentions_insta || args.iter().any(|word| dynamic(word, command.dynamic)) => {
                        Some(Check::Unknown("invokes a shell with an opaque script"))
                    }
                    _ => None,
                },
                _ => None,
            }
        };
        if let Some(result) = result {
            checks.push(result);
        }
    }
    for (_, _, script) in parsed.substitutions {
        inspect_script(&script, depth + 1, checks);
    }
}

fn dynamic_invocation(command: &words::Command) -> bool {
    let Some((name, index)) = words::command_at(&command.words) else {
        return false;
    };
    dynamic(&command.words[index], command.dynamic)
        || ((matches!(name.as_str(), "cargo" | "cargo-insta" | "eval" | "source" | ".")
            || is_shell(&name))
            && command.words[index + 1..]
                .iter()
                .any(|word| dynamic(word, command.dynamic)))
}

fn is_shell(name: &str) -> bool {
    matches!(name, "bash" | "sh" | "zsh" | "dash" | "ash" | "ksh")
}

fn dynamic(word: &Word, command_dynamic: bool) -> bool {
    word.text.contains("${{")
        || word.expansion_may_split
        || word.bare_expansion.is_some()
        || word.quoted_expansion.is_some()
        || (command_dynamic && word.text.contains(['$', '`']))
}

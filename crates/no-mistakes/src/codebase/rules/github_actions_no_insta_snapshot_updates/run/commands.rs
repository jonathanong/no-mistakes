use crate::codebase::rules::github_actions_download_integrity::words::{self, Word};

const MAX_NESTING: usize = 4;

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
    if parsed.unterminated_heredoc || parsed.unterminated_quote || parsed.unterminated_substitution
    {
        checks.push(Check::Unknown("contains incomplete shell syntax"));
        return;
    }
    for command in parsed.commands {
        let Some((name, index)) = words::command_at(&command.words) else {
            if command.dynamic {
                checks.push(Check::Unknown("contains a dynamic shell command"));
            }
            continue;
        };
        let command_word = &command.words[index];
        let args = &command.words[index + 1..];
        let result = if dynamic(command_word, command.dynamic)
            || matches!(
                name.as_str(),
                "eval" | "source" | "." | "case" | "for" | "select"
            ) {
            Some(Check::Unknown("invokes a dynamic command"))
        } else {
            match name.as_str() {
                "cargo" => cargo(args, command.dynamic),
                "cargo-insta" => insta(args, command.dynamic),
                "bash" | "sh" | "zsh" | "dash" | "ash" | "ksh" => match args {
                    [flag, script, ..]
                        if flag.text == "-c" && !dynamic(script, command.dynamic) =>
                    {
                        inspect_nested(&script.text, depth, checks);
                        None
                    }
                    _ => Some(Check::Unknown("invokes a shell with an opaque script")),
                },
                _ => None,
            }
        };
        if let Some(result) = result {
            checks.push(result);
        }
    }
    for (_, script) in parsed.substitutions {
        inspect_nested(&script, depth, checks);
    }
}

fn inspect_nested(script: &str, depth: usize, checks: &mut Vec<Check>) {
    if depth < MAX_NESTING {
        inspect_script(script, depth + 1, checks);
    } else {
        checks.push(Check::Unknown(
            "contains shell scripts nested too deeply to check",
        ));
    }
}

fn cargo(mut args: &[Word], command_dynamic: bool) -> Option<Check> {
    if let Some((first, rest)) = args
        .split_first()
        .filter(|(first, _)| first.text.starts_with('+'))
    {
        if dynamic(first, command_dynamic) {
            return Some(Check::Unknown("selects a dynamic Cargo toolchain"));
        }
        args = rest;
    }
    let args = skip_options(args, command_dynamic, CargoMode::Cargo)?;
    let (first, rest) = args.split_first()?;
    if dynamic(first, command_dynamic) {
        return Some(Check::Unknown("passes a dynamic subcommand to Cargo"));
    }
    (first.text == "insta")
        .then(|| insta(rest, command_dynamic))
        .flatten()
}

fn insta(args: &[Word], command_dynamic: bool) -> Option<Check> {
    let unknown = Check::Unknown("passes dynamic arguments to cargo-insta");
    let args = skip_options(args, command_dynamic, CargoMode::Insta)?;
    let (subcommand, rest) = args.split_first()?;
    if dynamic(subcommand, command_dynamic) {
        return Some(unknown);
    }
    let rest = rest
        .split(|word| word.text == "--")
        .next()
        .unwrap_or_default();
    if rest
        .iter()
        .any(|word| matches!(word.text.as_str(), "--help" | "-h"))
    {
        return None;
    }
    match subcommand.text.as_str() {
        "accept" | "approve" => Some(Check::Accept("accept")),
        "test" => {
            for word in rest {
                if dynamic(word, command_dynamic) {
                    return Some(unknown);
                }
                let flag = match word.text.as_str() {
                    "--accept" => "--accept",
                    "--accept-unseen" => "--accept-unseen",
                    "--force-update-snapshots" => "--force-update-snapshots",
                    text if matches!(
                        text.trim_end_matches([')', '}']),
                        "--accept" | "--accept-unseen" | "--force-update-snapshots"
                    ) =>
                    {
                        return Some(Check::Unknown(
                            "uses an ambiguous grouped cargo-insta argument",
                        ))
                    }
                    _ => continue,
                };
                return Some(Check::Accept(flag));
            }
            None
        }
        text if matches!(text.trim_end_matches([')', '}']), "accept" | "approve") => Some(
            Check::Unknown("uses an ambiguous grouped cargo-insta subcommand"),
        ),
        _ => None,
    }
}

enum CargoMode {
    Cargo,
    Insta,
}

fn skip_options(mut args: &[Word], command_dynamic: bool, mode: CargoMode) -> Option<&[Word]> {
    while let [word, rest @ ..] = args {
        let text = word.text.as_str();
        match text {
            "-h" | "--help" | "-V" | "--version" | "--" => return None,
            "--color" => args = rest.get(1..)?,
            "-C" | "--config" | "-Z" if matches!(mode, CargoMode::Cargo) => args = rest.get(1..)?,
            option if option.starts_with('-') && !dynamic(word, command_dynamic) => args = rest,
            _ => break,
        }
    }
    Some(args)
}

fn dynamic(word: &Word, command_dynamic: bool) -> bool {
    word.expansion_may_split
        || word.bare_expansion.is_some()
        || word.quoted_expansion.is_some()
        || (command_dynamic && word.text.contains(['$', '`']))
}

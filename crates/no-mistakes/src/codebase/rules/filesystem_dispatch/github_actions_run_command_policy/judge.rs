use super::matcher::{self, MatchOutcome};
use super::options::CompiledOptions;
use crate::codebase::rules::github_actions_download_integrity::words::{self, Word};

mod dynamic;
mod shell_c;

const MAX_NESTED_DEPTH: usize = 4;

#[derive(Debug)]
pub(super) struct Hit {
    pub(super) line: usize,
    pub(super) kind: HitKind,
}

#[derive(Debug)]
pub(super) enum HitKind {
    MissingRequired {
        command: String,
        required: String,
        policy_message: Option<String>,
    },
    MissingAnyRequired {
        command: String,
        alternatives: Vec<String>,
        policy_message: Option<String>,
    },
    ForbiddenArg {
        command: String,
        argument: String,
        pattern: String,
        policy_message: Option<String>,
    },
    Unanalyzable {
        token: String,
        detail: String,
    },
}

impl Hit {
    pub(super) fn token(&self) -> Option<&str> {
        match &self.kind {
            HitKind::MissingRequired { command, .. }
            | HitKind::MissingAnyRequired { command, .. } => command.split_whitespace().next(),
            HitKind::ForbiddenArg { argument, .. } => Some(argument.as_str()),
            HitKind::Unanalyzable { token, .. } => (!token.is_empty()).then_some(token.as_str()),
        }
    }
}

pub(super) fn scan_script(script: &str, compiled: &CompiledOptions) -> Vec<Hit> {
    let mut hits = Vec::new();
    scan_nested(script, compiled, 0, 0, &mut hits);
    hits.sort_by_key(|hit| hit.line);
    hits
}

fn scan_nested(
    script: &str,
    compiled: &CompiledOptions,
    line_offset: usize,
    depth: usize,
    hits: &mut Vec<Hit>,
) {
    let parsed = words::parse(script);
    if let Some(reason) = syntax_abstention(&parsed) {
        let detail = format!(
            "contains {reason}, so configured command policies could not be checked. Fix the shell syntax in this step."
        );
        hits.push(unanalyzable(1 + line_offset, String::new(), detail));
        return;
    }
    for command in &parsed.commands {
        let Some((name, at)) = words::command_at(&command.words) else {
            continue;
        };
        let cmd_word = &command.words[at];
        let cmd_line = cmd_word.line + line_offset;
        if matcher::is_dynamic_word(cmd_word, command.dynamic) {
            if dynamic::abstains_on_literal_script_path(&cmd_word.text, compiled) {
                continue;
            }
            let detail = format!(
                "invokes dynamic command `{}`, which cannot be statically checked against configured command policies. Call the command directly with a literal name.",
                cmd_word.text
            );
            hits.push(unanalyzable(cmd_line, cmd_word.text.clone(), detail));
            continue;
        }
        if name == "eval" {
            hits.push(unanalyzable(
                cmd_line,
                "eval".to_string(),
                "invokes `eval`, which cannot be statically checked against configured command policies. Replace `eval` with a direct command invocation.".to_string(),
            ));
            continue;
        }
        if shell_c::try_scan_shell_c(&name, command, at, line_offset, depth, compiled, hits) {
            continue;
        }
        check_command(
            &command.words,
            at,
            command.dynamic,
            line_offset,
            compiled,
            hits,
        );
    }
    for (sub_line, _, body) in &parsed.substitutions {
        scan_nested_body(
            body,
            line_offset + sub_line,
            "a command substitution",
            depth,
            compiled,
            hits,
        );
    }
}

fn scan_nested_body(
    body: &str,
    line: usize,
    kind: &str,
    depth: usize,
    compiled: &CompiledOptions,
    hits: &mut Vec<Hit>,
) {
    if depth < MAX_NESTED_DEPTH {
        scan_nested(body, compiled, line.saturating_sub(1), depth + 1, hits);
    } else {
        let token = body.lines().next().unwrap_or_default().trim().to_owned();
        let detail = format!(
            "exceeds the shell nesting limit at {kind}, so configured command policies could not be checked there. Move the nested commands to a direct run step."
        );
        hits.push(unanalyzable(line, token, detail));
    }
}

pub(super) fn unanalyzable(line: usize, token: String, detail: String) -> Hit {
    Hit {
        line,
        kind: HitKind::Unanalyzable { token, detail },
    }
}

fn check_command(
    words: &[Word],
    at: usize,
    command_dynamic: bool,
    line_offset: usize,
    compiled: &CompiledOptions,
    hits: &mut Vec<Hit>,
) {
    let mut dynamic_subcommand_reported = false;
    for policy in &compiled.commands {
        match matcher::match_command(words, at, command_dynamic, policy) {
            MatchOutcome::NoMatch => {}
            MatchOutcome::DynamicSubcommand { token, line } => {
                if !dynamic_subcommand_reported {
                    dynamic_subcommand_reported = true;
                    let detail = format!(
                        "passes dynamic subcommand `{token}` to `{}`, which cannot be statically checked against configured command policies. Use a literal subcommand name.",
                        policy.command[0]
                    );
                    hits.push(unanalyzable(line + line_offset, token, detail));
                }
            }
            MatchOutcome::Matched(inv) => {
                for mut hit in matcher::evaluate_policy(&inv, policy) {
                    hit.line += line_offset;
                    hits.push(hit);
                }
            }
        }
    }
}

fn syntax_abstention(parsed: &words::Parse) -> Option<&'static str> {
    if parsed.unterminated_heredoc {
        return Some("an unterminated here-document");
    }
    if parsed.unterminated_quote {
        return Some("an unterminated quote");
    }
    parsed
        .unterminated_substitution
        .then_some("an unterminated shell expansion")
}

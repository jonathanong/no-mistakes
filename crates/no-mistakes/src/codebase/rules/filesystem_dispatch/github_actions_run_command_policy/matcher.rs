use super::judge::{Hit, HitKind};
use super::options::{ArgScope, CompiledPolicy};
use crate::codebase::rules::github_actions_download_integrity::words::{self, Word};

mod args;
use args::{extract_positionals, is_option_token, matches_invocation, takes_value, unwrap_group};

pub(super) enum MatchOutcome<'a> {
    NoMatch,
    DynamicSubcommand { token: String, line: usize },
    Matched(ParsedInvocation<'a>),
}

pub(super) struct ParsedInvocation<'a> {
    pub(super) cmd_line: usize,
    pub(super) global_args: Vec<&'a Word>,
    pub(super) subcommand_args: Vec<&'a Word>,
    pub(super) positionals: Vec<&'a Word>,
    pub(super) splitting_word: Option<(String, usize)>,
}

pub(super) fn match_command<'a>(
    words: &'a [Word],
    cmd_at: usize,
    command_dynamic: bool,
    policy: &CompiledPolicy,
) -> MatchOutcome<'a> {
    let base = words::basename(unwrap_group(&words[cmd_at].text));
    if base != policy.command[0] {
        return MatchOutcome::NoMatch;
    }
    let mut global_args = Vec::new();
    let mut idx = cmd_at + 1;
    let mut sub_idx = 1;
    while idx < words.len() && sub_idx < policy.command.len() {
        let word = &words[idx];
        if word.text == "--" {
            return MatchOutcome::NoMatch;
        }
        if is_option_token(&word.text) {
            global_args.push(word);
            if !word.text.contains('=') && takes_value(&word.text, &policy.value_flags) {
                idx += 1;
                if let Some(value) = words.get(idx) {
                    global_args.push(value);
                }
            }
            idx += 1;
            continue;
        }
        if is_dynamic_word(word, command_dynamic) {
            return MatchOutcome::DynamicSubcommand {
                token: word.text.clone(),
                line: word.line,
            };
        }
        if word.text == policy.command[sub_idx] {
            sub_idx += 1;
            idx += 1;
        } else {
            return MatchOutcome::NoMatch;
        }
    }
    if sub_idx < policy.command.len() {
        return MatchOutcome::NoMatch;
    }
    let subcommand_args: Vec<&Word> = words[idx..].iter().collect();
    let positionals = extract_positionals(&subcommand_args, &policy.value_flags);
    let splitting_word = global_args
        .iter()
        .chain(subcommand_args.iter())
        .copied()
        .find(|word| is_word_splitting_expansion(word))
        .map(|word| (word.text.clone(), word.line));
    MatchOutcome::Matched(ParsedInvocation {
        cmd_line: words[cmd_at].line,
        global_args,
        subcommand_args,
        positionals,
        splitting_word,
    })
}

pub(super) fn evaluate_policy(inv: &ParsedInvocation<'_>, policy: &CompiledPolicy) -> Vec<Hit> {
    let mut hits = Vec::new();
    if !policy.when_any_args.is_empty() {
        let triggered = policy
            .when_any_args
            .iter()
            .any(|matcher| matches_invocation(inv, matcher, &policy.value_flags));
        if !triggered {
            if let Some((token, line)) = &inv.splitting_word {
                hits.push(splitting_abstention(*line, token, &policy.label));
            }
            return hits;
        }
    }
    let mut deferred_split = false;
    for required in &policy.require_args {
        if !matches_invocation(inv, required, &policy.value_flags) {
            if inv.splitting_word.is_some() {
                deferred_split = true;
            } else {
                hits.push(Hit {
                    line: inv.cmd_line,
                    kind: HitKind::MissingRequired {
                        command: policy.label.clone(),
                        required: required.label.clone(),
                        policy_message: policy.message.clone(),
                    },
                });
            }
        }
    }
    if !policy.require_any_args.is_empty()
        && !policy
            .require_any_args
            .iter()
            .any(|alt| matches_invocation(inv, alt, &policy.value_flags))
    {
        if inv.splitting_word.is_some() {
            deferred_split = true;
        } else {
            hits.push(Hit {
                line: inv.cmd_line,
                kind: HitKind::MissingAnyRequired {
                    command: policy.label.clone(),
                    alternatives: policy
                        .require_any_args
                        .iter()
                        .map(|alt| alt.label.clone())
                        .collect(),
                    policy_message: policy.message.clone(),
                },
            });
        }
    }
    if deferred_split {
        if let Some((token, line)) = &inv.splitting_word {
            hits.push(splitting_abstention(*line, token, &policy.label));
        }
    }
    for forbidden in &policy.forbid_arg_patterns {
        let targets: Vec<&Word> = match forbidden.scope {
            ArgScope::Any => inv
                .global_args
                .iter()
                .chain(inv.subcommand_args.iter())
                .copied()
                .collect(),
            ArgScope::Positional => inv.positionals.clone(),
        };
        for word in targets {
            if forbidden.regex.is_match(&word.text) {
                hits.push(Hit {
                    line: word.line,
                    kind: HitKind::ForbiddenArg {
                        command: policy.label.clone(),
                        argument: word.text.clone(),
                        pattern: forbidden.raw.clone(),
                        policy_message: forbidden
                            .message
                            .clone()
                            .or_else(|| policy.message.clone()),
                    },
                });
            }
        }
    }
    hits
}

pub(super) fn is_dynamic_word(word: &Word, command_dynamic: bool) -> bool {
    word.bare_expansion.is_some()
        || word.quoted_expansion.is_some()
        || word.expansion_may_split
        || word.text.contains("${{")
        || (command_dynamic && (word.text.contains('$') || word.text.contains('`')))
}

fn is_word_splitting_expansion(word: &Word) -> bool {
    word.expansion_may_split
        || word.text.contains("$@")
        || word.text.contains("$*")
        || word.text.contains("[@]}")
        || word.text.contains("[*]}")
}

fn splitting_abstention(line: usize, token: &str, command: &str) -> Hit {
    Hit {
        line,
        kind: HitKind::Unanalyzable {
            token: token.to_string(),
            detail: format!(
                "passes dynamic expansion `{token}` to `{command}`, so required arguments cannot be verified statically. Pass literal arguments or quote scalar values."
            ),
        },
    }
}

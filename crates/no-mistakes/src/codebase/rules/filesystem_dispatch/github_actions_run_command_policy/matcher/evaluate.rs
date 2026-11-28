use super::args::{is_option_token, match_invocation, ArgMatch};
use super::ParsedInvocation;
use crate::codebase::rules::filesystem_dispatch::github_actions_run_command_policy::judge::{
    Hit, HitKind,
};
use crate::codebase::rules::filesystem_dispatch::github_actions_run_command_policy::options::{
    ArgScope, CompiledPolicy,
};
use crate::codebase::rules::github_actions_download_integrity::words::Word;

pub(in super::super) fn evaluate_policy(
    inv: &ParsedInvocation<'_>,
    policy: &CompiledPolicy,
) -> Vec<Hit> {
    let uncertain_when = match check_when_gate(inv, policy) {
        UnlessGate::Exempt => return Vec::new(),
        UnlessGate::Apply(uncertain) => uncertain,
    };
    let uncertain_unless = match check_unless_gate(inv, policy) {
        UnlessGate::Exempt => return Vec::new(),
        UnlessGate::Apply(uncertain) => uncertain,
    };
    let mut hits = Vec::new();
    let uncertain_req = check_requirements(inv, policy, &mut hits);
    collect_forbidden_hits(inv, policy, &mut hits);
    if !hits.is_empty() {
        if let Some((token, line)) = uncertain_when.as_ref().or(uncertain_unless.as_ref()) {
            return vec![dynamic_abstention(*line, token, &policy.label)];
        }
    }
    if let Some((token, line)) = uncertain_req {
        hits.push(dynamic_abstention(line, &token, &policy.label));
    }
    hits
}

fn check_when_gate(inv: &ParsedInvocation<'_>, policy: &CompiledPolicy) -> UnlessGate {
    if policy.when_any_args.is_empty() {
        return UnlessGate::Apply(None);
    }
    let mut dynamic_trigger = None;
    for matcher in &policy.when_any_args {
        match match_invocation(inv, matcher, &policy.value_flags, false) {
            ArgMatch::Matched => return UnlessGate::Apply(None),
            ArgMatch::DynamicValue { token, line } => {
                dynamic_trigger.get_or_insert((token, line));
            }
            ArgMatch::NotMatched => {}
        }
    }
    match super::split_hit(inv, super::flags_only(&policy.when_any_args)).or(dynamic_trigger) {
        Some(hit) => UnlessGate::Apply(Some(hit)),
        None => UnlessGate::Exempt,
    }
}

enum UnlessGate {
    Exempt,
    Apply(Option<(String, usize)>),
}

fn check_unless_gate(inv: &ParsedInvocation<'_>, policy: &CompiledPolicy) -> UnlessGate {
    if policy.unless_any_args.is_empty() {
        return UnlessGate::Apply(None);
    }
    let mut uncertain = None;
    for matcher in &policy.unless_any_args {
        match match_invocation(inv, matcher, &policy.value_flags, false) {
            ArgMatch::Matched => return UnlessGate::Exempt,
            ArgMatch::DynamicValue { token, line } => {
                uncertain.get_or_insert((token, line));
            }
            ArgMatch::NotMatched => {}
        }
    }
    UnlessGate::Apply(
        uncertain.or_else(|| super::split_hit(inv, super::flags_only(&policy.unless_any_args))),
    )
}

fn check_requirements(
    inv: &ParsedInvocation<'_>,
    policy: &CompiledPolicy,
    hits: &mut Vec<Hit>,
) -> Option<(String, usize)> {
    let mut uncertain: Option<(String, usize)> = None;
    for required in &policy.require_args {
        match match_invocation(inv, required, &policy.value_flags, true) {
            ArgMatch::Matched => {}
            ArgMatch::DynamicValue { token, line } => {
                uncertain.get_or_insert((token, line));
            }
            ArgMatch::NotMatched => {
                let flag = required
                    .tokens
                    .first()
                    .is_some_and(|token| is_option_token(token));
                if let Some((token, line)) = super::split_hit(inv, flag) {
                    uncertain.get_or_insert((token, line));
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
    }
    if !policy.require_any_args.is_empty() {
        let mut any_matched = false;
        let mut any_dynamic = None;
        for alt in &policy.require_any_args {
            match match_invocation(inv, alt, &policy.value_flags, true) {
                ArgMatch::Matched => {
                    any_matched = true;
                    break;
                }
                ArgMatch::DynamicValue { token, line } => {
                    any_dynamic.get_or_insert((token, line));
                }
                ArgMatch::NotMatched => {}
            }
        }
        if !any_matched {
            let split = super::split_hit(inv, super::flags_only(&policy.require_any_args));
            if let Some((token, line)) = any_dynamic.or(split) {
                uncertain.get_or_insert((token, line));
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
    }
    uncertain
}

fn collect_forbidden_hits(
    inv: &ParsedInvocation<'_>,
    policy: &CompiledPolicy,
    hits: &mut Vec<Hit>,
) {
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
}

fn dynamic_abstention(line: usize, token: &str, command: &str) -> Hit {
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

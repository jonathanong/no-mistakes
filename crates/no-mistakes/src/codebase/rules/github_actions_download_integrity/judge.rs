use super::assign::prepare_max_script;
use super::env::UrlBindings;
use super::url;
use super::verify;
use super::words;
use std::cell::OnceCell;

mod command;
mod operands;
mod options;

pub(super) struct Hit {
    pub(super) url: String,
    pub(super) line: usize,
    pub(super) command: String,
    pub(super) unanalyzable: bool,
    pub(super) indirect_command: bool,
}

pub(super) fn scan_script(
    script: &str,
    helpers: &[String],
    low: bool,
    strict: bool,
    env: Option<&serde_yaml::Mapping>,
) -> Vec<Hit> {
    if url::has_skip(script) {
        return Vec::new();
    }
    let raw = words::parse_ordered(script, strict);
    if !strict && raw.commands.iter().any(verify::command_verifies) {
        return Vec::new();
    }
    let expanded = (!low).then(|| prepare_max_script(script)).flatten();
    let env = env
        .filter(|env| !low && !env.is_empty())
        .and_then(|env| UrlBindings::new(env, script, &raw));
    let (parsed, original) = match &expanded {
        Some(expanded) => (words::parse_ordered(&expanded.text, strict), Some(raw)),
        None => (raw, None),
    };
    if parsed.unterminated_heredoc {
        return Vec::new();
    }
    let mut hits = Vec::new();
    let binding = OnceCell::new();
    let mut pending_start = 0;
    for mut command in parsed.commands {
        let raw_command = match (&expanded, &original) {
            (Some(expanded), Some(raw)) => {
                expanded.original_anchor(command.anchor).and_then(|anchor| {
                    let index = raw
                        .commands
                        .binary_search_by_key(&anchor, |command| command.anchor)
                        .ok()?;
                    raw.commands.get(index)
                })
            }
            _ => None,
        };
        let original_command = if expanded.is_some() {
            raw_command
        } else {
            Some(&command)
        };
        if strict
            && original_command.is_some_and(verify::command_verifies)
            && verify::command_verifies(&command)
        {
            hits.truncate(pending_start);
            continue;
        }
        let Some((mut name, mut arg_at)) = words::command_at(&command.words) else {
            continue;
        };
        let mut indirect_command = false;
        if !words::is_downloader(&name, helpers) {
            if !low || !name.contains('$') {
                continue;
            }
            let Some(recovered) =
                command::recover_downloader(&mut command, script, helpers, &binding)
            else {
                continue;
            };
            (name, arg_at) = recovered;
            indirect_command = true;
        }
        let args = command.words.get(arg_at + 1..).unwrap_or(&[]);
        let candidates = operands::url_words(&name, args);
        // An expansion can become an option and consume a later literal URL.
        // Judge the command's argument shape before claiming any unsafe URL.
        if low {
            let unknown = options::dynamic_option(&name, args)
                .or_else(|| {
                    candidates
                        .iter()
                        .find(|operand| dynamic(operand.text))
                        .map(|operand| (operand.word, operand.text))
                })
                .or_else(|| {
                    if indirect_command {
                        candidates
                            .iter()
                            .find(|operand| url::http_url(operand.text).is_some())
                            .map(|operand| (operand.word, operand.text))
                    } else {
                        None
                    }
                });
            if let Some((word, url)) = unknown {
                if strict {
                    pending_start = hits.len();
                }
                hits.push(Hit {
                    url: url.to_string(),
                    line: word.line,
                    command: name,
                    unanalyzable: true,
                    indirect_command,
                });
                continue;
            }
        }
        let env = env.as_ref().and_then(|bindings| {
            let original = if expanded.is_some() {
                raw_command?
            } else {
                &command
            };
            let shape = operands::original_shape(original)?;
            shape
                .matches(&name, arg_at, args.len())
                .then_some((bindings, shape))
        });
        let mut unknown_role = env.is_some() && options::dynamic_option(&name, args).is_some();
        for operand in candidates {
            let word = operand.word;
            if indirect_command {
                continue;
            }
            let url = env
                .as_ref()
                .filter(|(_, shape)| !unknown_role && shape.same_operand(&operand))
                .and_then(|(env, _)| env.resolve(word, operand.text))
                .unwrap_or(operand.text);
            unknown_role |= operand.may_be_option && dynamic(url);
            let Some(parsed) = url::http_url(url) else {
                if strict && dynamic(url) {
                    pending_start = hits.len();
                }
                continue;
            };
            if parsed.checksum {
                continue;
            }
            if strict {
                pending_start = hits.len();
            }
            hits.push(Hit {
                url: url.to_string(),
                line: word.line,
                command: name.clone(),
                unanalyzable: false,
                indirect_command: false,
            });
        }
    }
    hits
}

fn dynamic(value: &str) -> bool {
    value.contains(['$', '`', '{'])
}

pub(super) fn filter_trusted(hits: Vec<Hit>, hosts: &[String]) -> Vec<Hit> {
    hits.into_iter()
        .filter(|hit| {
            hit.unanalyzable
                || url::http_url(&hit.url).is_none_or(|url| !url::trusted(&url.host, hosts))
        })
        .collect()
}

pub(super) use url::{configured_host, has_skip};

use super::assign::prepare_max_script;
use super::env::UrlBindings;
use super::url::{self, UrlHost};
use super::verify;
use super::words::{self, Word};
use std::cell::OnceCell;

mod command;
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
    let (parsed, original) = match expanded {
        Some(expanded) => (words::parse_ordered(&expanded, strict), Some(raw)),
        None => (raw, None),
    };
    if parsed.unterminated_heredoc {
        return Vec::new();
    }
    let mut hits = Vec::new();
    let binding = OnceCell::new();
    let mut pending_start = 0;
    let command_count = parsed.commands.len();
    for (index, mut command) in parsed.commands.into_iter().enumerate() {
        let original_command = match &original {
            Some(raw) if raw.commands.len() == command_count => raw.commands.get(index),
            Some(_) => None,
            None => Some(&command),
        };
        if strict
            && original_command.is_some_and(verify::command_verifies)
            && verify::command_verifies(&command)
        {
            hits.truncate(pending_start);
            continue;
        }
        let env = env
            .as_ref()
            .filter(|_| original_command.is_some_and(|raw| raw.words == command.words));
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
        let candidates = url_words(&name, args);
        // An expansion can become an option and consume a later literal URL.
        // Judge the command's argument shape before claiming any unsafe URL.
        if low {
            let unknown = options::dynamic_option(&name, args)
                .or_else(|| candidates.iter().copied().find(|(_, url)| dynamic(url)))
                .or_else(|| {
                    if indirect_command {
                        candidates
                            .iter()
                            .copied()
                            .find(|(_, url)| url::http_url(url).is_some())
                    } else {
                        None
                    }
                });
            if let Some((word, url)) = unknown {
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
        let next_pending = hits.len();
        for (word, url) in candidates {
            if indirect_command {
                continue;
            }
            let url = env.and_then(|env| env.resolve(word, url)).unwrap_or(url);
            let Some(parsed) = url::http_url(url) else {
                continue;
            };
            if parsed.checksum {
                continue;
            }
            if strict {
                pending_start = next_pending;
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
            hit.unanalyzable || url::http_url(&hit.url).is_none_or(|url| !trusted_host(&url, hosts))
        })
        .collect()
}

fn trusted_host(url: &UrlHost, hosts: &[String]) -> bool {
    url::trusted(&url.host, hosts)
}

fn url_words<'a>(command: &str, args: &'a [Word]) -> Vec<(&'a Word, &'a str)> {
    let mut out = Vec::new();
    let mut index = 0;
    while index < args.len() {
        let token = args[index].text.as_str();
        if token == "--" {
            for word in &args[index + 1..] {
                out.push((word, word.text.as_str()));
            }
            break;
        }
        if let Some(inline) = token.strip_prefix("--url=") {
            out.push((&args[index], inline));
            index += 1;
            continue;
        }
        if verify::takes_url_slot(command, token) {
            index += 2;
            continue;
        }
        if token.starts_with('-') && token != "-" {
            index += 1;
            continue;
        }
        out.push((&args[index], token));
        index += 1;
    }
    out
}

pub(super) use url::{configured_host, has_skip};

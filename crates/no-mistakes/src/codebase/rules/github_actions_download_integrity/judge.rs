use super::assign::expand_uppercase;
use super::url::{self, UrlHost};
use super::verify;
use super::words::{self, Word};

pub(super) struct Hit {
    pub(super) url: String,
    pub(super) line: usize,
    pub(super) command: String,
}

pub(super) fn download_hits(script: &str, helpers: &[String]) -> Vec<Hit> {
    if words::has_heredoc(script) || url::has_skip(script) || verify::step_verifies(script) {
        return Vec::new();
    }
    let expanded = expand_uppercase(script);
    let mut hits = Vec::new();
    for command in words::commands(&expanded) {
        if command.dynamic {
            continue;
        }
        let Some(name) = words::command_name(&command.words) else {
            continue;
        };
        if !words::is_downloader(&name, helpers) {
            continue;
        }
        for (word, url) in url_words(&name, &command.words) {
            let Some(parsed) = url::http_url(url) else {
                continue;
            };
            if parsed.checksum {
                continue;
            }
            hits.push(Hit {
                url: url.to_string(),
                line: word.line,
                command: name.clone(),
            });
        }
    }
    hits
}

pub(super) fn filter_trusted(hits: Vec<Hit>, hosts: &[String]) -> Vec<Hit> {
    hits.into_iter()
        .filter(|hit| url::http_url(&hit.url).is_none_or(|url| !trusted_host(&url, hosts)))
        .collect()
}

fn trusted_host(url: &UrlHost, hosts: &[String]) -> bool {
    url::trusted(&url.host, hosts)
}

fn url_words<'a>(command: &str, words: &'a [Word]) -> Vec<(&'a Word, &'a str)> {
    let mut out = Vec::new();
    let mut index = 0;
    let args = words
        .iter()
        .position(|word| !words::is_assignment(&word.text))
        .map(|start| &words[start + 1..])
        .unwrap_or(&[]);
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

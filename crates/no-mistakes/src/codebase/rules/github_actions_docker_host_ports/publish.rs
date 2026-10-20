use super::flags::{basename, long_arg, short_kind};
use super::mapping::{classify, Verdict};
use super::shell::{commands, Word};

pub(super) struct Hit {
    pub(super) spec: String,
    pub(super) host: String,
    pub(super) line: usize,
}

mod lead;
mod wrappers;

pub(super) fn publish_hits(script: &str) -> Vec<Hit> {
    let mut hits = Vec::new();
    for command in commands(script) {
        if command.dynamic {
            continue;
        }
        hits.extend(command_hits(&command.words));
    }
    hits
}

fn command_hits(words: &[Word]) -> Vec<Hit> {
    let index = lead::skip_prefix(words);
    if let Some(operand) = wrappers::shell_operand(words, index) {
        let base = operand.line.saturating_sub(1);
        return publish_hits(&operand.text)
            .into_iter()
            .map(|mut hit| {
                hit.line += base;
                hit
            })
            .collect();
    }
    docker_hits(words).unwrap_or_default()
}

fn docker_hits(words: &[Word]) -> Option<Vec<Hit>> {
    let mut index = lead::skip_prefix(words);
    if basename(lead::command_text(words.get(index)?.text.as_str())) != "docker" {
        return Some(Vec::new());
    }
    index += 1;
    index = lead::skip_globals(words, index);
    if words.get(index).map(|word| word.text.as_str()) == Some("container") {
        index += 1;
    }
    if !matches!(
        words.get(index).map(|word| word.text.as_str()),
        Some("run" | "create")
    ) {
        return Some(Vec::new());
    }
    index += 1;
    let mut hits = Vec::new();
    while index < words.len() {
        let word = &words[index];
        if word.dynamic || word.text == "--" || !word.text.starts_with('-') {
            break;
        }
        if let Some(cluster) = word
            .text
            .strip_prefix('-')
            .filter(|text| !text.starts_with('-'))
        {
            let Some(taken) = take_short(cluster, words, index) else {
                break;
            };
            if let Some(spec) = taken.publish {
                note(&mut hits, spec, taken.line);
            }
            index += taken.words;
            continue;
        }
        let Some(taken) = take_long(word, words, index) else {
            if word.text.contains('=') {
                index += 1;
                continue;
            }
            break;
        };
        if let Some(spec) = taken.publish {
            note(&mut hits, spec, taken.line);
        }
        index += taken.words;
    }
    Some(hits)
}

struct Taken {
    words: usize,
    publish: Option<String>,
    line: usize,
}

fn note(hits: &mut Vec<Hit>, spec: String, line: usize) {
    if let Verdict::Literal(host) = classify(&spec) {
        hits.push(Hit { spec, host, line });
    }
}

fn take_short(cluster: &str, words: &[Word], index: usize) -> Option<Taken> {
    let chars: Vec<char> = cluster.chars().collect();
    let mut pos = 0;
    while pos < chars.len() {
        let kind = short_kind(chars[pos])?;
        pos += 1;
        if !kind {
            continue;
        }
        let rest: String = chars[pos..].iter().collect();
        if rest.is_empty() {
            let next = words.get(index + 1)?;
            if next.dynamic {
                return Some(Taken {
                    words: 2,
                    publish: None,
                    line: next.line,
                });
            }
            return Some(publish_taken(
                chars[pos - 1] == 'p',
                2,
                next.text.clone(),
                next.line,
            ));
        }
        let spec = rest.strip_prefix('=').unwrap_or(&rest);
        return Some(publish_taken(
            chars[pos - 1] == 'p',
            1,
            spec.to_string(),
            words[index].line,
        ));
    }
    Some(Taken {
        words: 1,
        publish: None,
        line: 0,
    })
}

fn take_long(word: &Word, words: &[Word], index: usize) -> Option<Taken> {
    let body = word.text.strip_prefix("--")?;
    let (name, inline) = body
        .split_once('=')
        .map(|(name, value)| (name, Some(value)))
        .unwrap_or((body, None));
    let takes_value = long_arg(name)?;
    if !takes_value {
        if inline.is_some() {
            return None;
        }
        return Some(Taken {
            words: 1,
            publish: None,
            line: 0,
        });
    }
    let (spec, line, words_used) = if let Some(value) = inline {
        (value.to_string(), word.line, 1)
    } else {
        let next = words.get(index + 1)?;
        if next.dynamic {
            return Some(Taken {
                words: 2,
                publish: None,
                line: next.line,
            });
        }
        (next.text.clone(), next.line, 2)
    };
    Some(publish_taken(name == "publish", words_used, spec, line))
}

fn publish_taken(publish: bool, words: usize, spec: String, line: usize) -> Taken {
    Taken {
        words,
        publish: publish.then_some(spec),
        line,
    }
}

use super::lex::{assignment, command_words, identifier, Word};

pub(super) struct Targets {
    pub names: Vec<String>,
    pub scalars: Vec<String>,
    pub keep: bool,
    pub callback: Option<Option<String>>,
}

pub(super) fn targets(words: &[Word<'_>]) -> Option<Targets> {
    let mut command = command_words(words).skip_while(|word| assignment(word.raw).is_some());
    let mapfile = match command.next()?.raw {
        "read" => false,
        "mapfile" | "readarray" => true,
        _ => return None,
    };
    let takes = if mapfile { "dnOsuCc" } else { "adinNptu" };
    let (mut names, mut positional, mut keep) = (Vec::new(), Vec::new(), false);
    let (mut has_array_opt, mut callback) = (false, None);
    while let Some(word) = command.next() {
        if word.raw == "--" {
            positional.extend(command.map(|word| word.raw));
            break;
        }
        let Some(letters) = word.raw.strip_prefix('-').filter(|flags| !flags.is_empty()) else {
            positional.push(word.raw);
            continue;
        };
        let Some((at, letter)) = letters.char_indices().find(|(_, ch)| takes.contains(*ch)) else {
            continue;
        };
        let rest = &letters[at + 1..];
        let argument = if rest.is_empty() {
            command.next().map(|word| word.raw)
        } else {
            Some(rest)
        };
        keep |= letter == 'O';
        if letter == 'a' && !mapfile {
            has_array_opt = true;
            names.extend(argument);
        } else if letter == 'C' && mapfile {
            callback = argument.and_then(decode_callback);
        }
    }
    let scalars = if mapfile || has_array_opt {
        if mapfile {
            names = vec![positional.first().copied().unwrap_or("MAPFILE")];
        }
        Vec::new()
    } else if positional.is_empty() {
        vec!["REPLY".to_owned()]
    } else {
        positional
            .into_iter()
            .filter_map(decode_scalar_target)
            .collect()
    };
    let names = names.into_iter().filter_map(decode_target).collect();
    Some(Targets {
        names,
        scalars,
        keep,
        callback,
    })
}

fn decode_callback(raw: &str) -> Option<Option<String>> {
    match decode_unquoted(raw) {
        Some((decoded, false)) if decoded.is_empty() => None,
        Some((decoded, false)) if identifier(&decoded) => Some(Some(decoded)),
        _ => Some(None),
    }
}

#[derive(Clone, Copy)]
enum Quote {
    Single,
    Double,
}

fn decode_unquoted(raw: &str) -> Option<(String, bool)> {
    let mut out = String::with_capacity(raw.len());
    let mut chars = raw.chars();
    let mut quote = None;
    let mut expanded = false;
    while let Some(ch) = chars.next() {
        match (quote, ch) {
            (Some(Quote::Single), '\'') | (Some(Quote::Double), '"') => quote = None,
            (Some(Quote::Double), '\\') => {
                let next = chars.next()?;
                if matches!(next, '$' | '`' | '"' | '\\') {
                    out.push(next);
                } else if next != '\n' {
                    out.push('\\');
                    out.push(next);
                }
            }
            (None, '\'') => quote = Some(Quote::Single),
            (None, '"') => quote = Some(Quote::Double),
            (None, '\\') => {
                let next = chars.next()?;
                if next != '\n' {
                    out.push(next);
                }
            }
            (Some(Quote::Double) | None, '$' | '`') => {
                expanded = true;
                out.push(ch);
            }
            _ => out.push(ch),
        }
    }
    quote.is_none().then_some((out, expanded))
}

fn decode_target(raw: &str) -> Option<String> {
    let (out, _) = decode_unquoted(raw)?;
    identifier(&out).then_some(out)
}

fn decode_scalar_target(raw: &str) -> Option<String> {
    let (out, _) = decode_unquoted(raw)?;
    let base = out
        .strip_suffix(']')
        .and_then(|s| s.split_once('['))
        .map_or(out.as_str(), |(n, _)| n);
    identifier(base).then(|| base.to_owned())
}

#[cfg(test)]
mod tests;

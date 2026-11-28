use super::lex::{assignment, command_words, identifier, Word};

pub(super) struct Targets {
    pub names: Vec<String>,
    pub keep: bool,
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
            names.extend(argument);
        }
    }
    if mapfile {
        names = vec![positional.first().copied().unwrap_or("MAPFILE")];
    }
    let names = names.into_iter().filter_map(decode_target).collect();
    Some(Targets { names, keep })
}

#[derive(Clone, Copy)]
enum Quote {
    Single,
    Double,
}

fn decode_target(raw: &str) -> Option<String> {
    let mut out = String::with_capacity(raw.len());
    let mut chars = raw.chars();
    let mut quote = None;
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
            _ => out.push(ch),
        }
    }
    (quote.is_none() && identifier(&out)).then_some(out)
}

#[cfg(test)]
mod tests;

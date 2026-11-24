use super::lex::{assignment, command_words, identifier, Word};

pub(super) struct Targets<'a> {
    pub names: Vec<&'a str>,
    pub keep: bool,
}

// `read -a` and `mapfile`/`readarray` fill an array from input that may be empty.
pub(super) fn targets<'a>(words: &[Word<'a>]) -> Option<Targets<'a>> {
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
    names.retain(|name| identifier(name));
    Some(Targets { names, keep })
}

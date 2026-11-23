use super::lex::{assignment, command_words, identifier, Lexer, Word};
use crate::fx::FxHashMap;

pub const RULE_ID: &str = "bash-empty-array-nounset";

pub(super) fn scan(source: &str) -> Vec<(&str, usize)> {
    let mut arrays = FxHashMap::default();
    let mut active = false;
    let mut hits = Vec::new();
    Lexer::new(source).commands(|words, _| {
        if active {
            hits.extend(
                words
                    .iter()
                    .flat_map(|word| word.refs.iter().copied())
                    .filter(|(name, _)| arrays.get(name) == Some(&false)),
            );
        }
        let mut command = command_words(words).peekable();
        let Some(first) = command.peek() else {
            return;
        };
        match first.raw {
            "set" => active = nounset(command.skip(1), active),
            "unset" => {
                for word in command.skip(1) {
                    if let Some(nonempty) = arrays.get_mut(word.raw) {
                        *nonempty = false;
                    }
                }
            }
            "declare" | "local" => {
                command.next();
                let mut array = false;
                while let Some((enable, flags)) = command.peek().and_then(|word| flag(word.raw)) {
                    if flags.contains('a') {
                        array = enable;
                    }
                    command.next();
                }
                command.next_if(|word| word.raw == "--");
                for word in command {
                    if array && identifier(word.raw) {
                        arrays.entry(word.raw).or_insert(false);
                    }
                    remember_array(&mut arrays, word, array);
                }
            }
            _ => {
                for word in command.take_while(|word| assignment(word.raw).is_some()) {
                    remember_array(&mut arrays, word, false);
                }
            }
        }
    });
    hits
}

fn remember_array<'a>(arrays: &mut FxHashMap<&'a str, bool>, word: &Word<'a>, declared: bool) {
    let Some((name, append)) = assignment(word.raw) else {
        return;
    };
    if let Some(literal) = word.array_literal {
        let nonempty = arrays.entry(name).or_default();
        *nonempty = literal || (append && *nonempty);
    } else if declared {
        arrays.insert(name, word.is_literal());
    }
}

fn nounset<'w, 'a: 'w>(mut words: impl Iterator<Item = &'w Word<'a>>, mut active: bool) -> bool {
    while let Some(word) = words.next() {
        let Some((enable, flags)) = flag(word.raw) else {
            break;
        };
        if flags.contains('u') {
            active = enable;
        }
        if flags.contains('o') && words.next().is_some_and(|option| option.raw == "nounset") {
            active = enable;
        }
    }
    active
}

fn flag(word: &str) -> Option<(bool, &str)> {
    let (enable, options) = if let Some(options) = word.strip_prefix('-') {
        (true, options)
    } else {
        (false, word.strip_prefix('+')?)
    };
    (!options.is_empty() && options.bytes().all(|byte| byte.is_ascii_alphabetic()))
        .then_some((enable, options))
}

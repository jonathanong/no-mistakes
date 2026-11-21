use super::super::command::Word;
use super::flags::{literal_value, takes_value, MODES};

pub(super) struct Parsed {
    pub(super) projects: Vec<String>,
    pub(super) positionals: Vec<Word>,
    pub(super) name_flags: Vec<Word>,
    pub(super) blocked: bool,
}

pub(super) fn parse(argv: &[Word]) -> Parsed {
    let mut parsed = Parsed {
        projects: Vec::new(),
        positionals: Vec::new(),
        name_flags: Vec::new(),
        blocked: false,
    };
    let mut index = 0;
    let mut ended = false;
    while index < argv.len() {
        let word = &argv[index];
        if ended || !word.text.starts_with('-') {
            index = positional(&mut parsed, argv, index, ended);
            ended = ended || word.text == "--";
            continue;
        }
        if word.text == "--" {
            ended = true;
            index += 1;
            continue;
        }
        index = take_flag(&mut parsed, argv, index);
    }
    parsed
}

fn positional(parsed: &mut Parsed, argv: &[Word], index: usize, ended: bool) -> usize {
    let word = &argv[index];
    if word.text == "--" {
        return index + 1;
    }
    if word.dynamic {
        parsed.blocked = true;
        return index + 1;
    }
    if !ended && parsed.positionals.is_empty() && MODES.contains(&word.text.as_str()) {
        return index + 1;
    }
    parsed.positionals.push(word.clone());
    index + 1
}

fn take_flag(parsed: &mut Parsed, argv: &[Word], index: usize) -> usize {
    let (name, inline) = split(&argv[index].text);
    let dynamic = argv[index].dynamic;
    if name == "--project" {
        return store_project(parsed, argv, index, inline, dynamic);
    }
    if name == "--testNamePattern" || name == "-t" {
        return store_name(parsed, argv, index, inline, dynamic);
    }
    if takes_value(name) {
        return if inline.is_some() {
            end_of_value(argv, index)
        } else {
            consume_value(argv, index)
        };
    }
    if inline.is_some() {
        parsed.blocked |= dynamic;
        return index + 1;
    }
    match argv.get(index + 1) {
        Some(next) if next.dynamic && !next.text.starts_with('-') => {
            parsed.blocked = true;
            end_of_value(argv, index + 1)
        }
        Some(next) if literal_value(&next.text) => index + 2,
        _ => index + 1,
    }
}

fn store_project(
    parsed: &mut Parsed,
    argv: &[Word],
    index: usize,
    inline: Option<&str>,
    dynamic: bool,
) -> usize {
    if let Some(value) = inline {
        if dynamic {
            parsed.blocked = true;
        } else if !value.is_empty() {
            parsed.projects.push(value.to_string());
        }
        return index + 1;
    }
    let Some(next) = argv
        .get(index + 1)
        .filter(|word| !word.text.starts_with('-'))
    else {
        return index + 1;
    };
    if next.dynamic {
        parsed.blocked = true;
    } else {
        parsed.projects.push(next.text.clone());
    }
    end_of_value(argv, index + 1)
}

fn store_name(
    parsed: &mut Parsed,
    argv: &[Word],
    index: usize,
    inline: Option<&str>,
    dynamic: bool,
) -> usize {
    if let Some(value) = inline {
        if !dynamic && !value.is_empty() {
            parsed.name_flags.push(Word {
                text: value.to_string(),
                line: argv[index].line,
                dynamic: false,
            });
        }
        return index + 1;
    }
    let Some(next) = argv
        .get(index + 1)
        .filter(|word| !word.text.starts_with('-'))
    else {
        return index + 1;
    };
    if !next.dynamic {
        parsed.name_flags.push(next.clone());
    }
    end_of_value(argv, index + 1)
}

fn consume_value(argv: &[Word], index: usize) -> usize {
    let takes_next = argv
        .get(index + 1)
        .is_some_and(|word| !word.text.starts_with('-') || word.dynamic);
    if takes_next {
        end_of_value(argv, index + 1)
    } else {
        index + 1
    }
}

fn end_of_value(argv: &[Word], value_at: usize) -> usize {
    let word = &argv[value_at];
    if !word.text.contains("${{") || word.text.contains("}}") {
        return value_at + 1;
    }
    let mut next = value_at + 1;
    while next < argv.len() && !argv[next].text.starts_with('-') {
        let end = argv[next].text.contains("}}");
        next += 1;
        if end {
            break;
        }
    }
    next
}

fn split(text: &str) -> (&str, Option<&str>) {
    text.split_once('=')
        .map_or((text, None), |(name, value)| (name, Some(value)))
}

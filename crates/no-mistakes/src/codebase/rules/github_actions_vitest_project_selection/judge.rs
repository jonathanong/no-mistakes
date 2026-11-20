use super::command::{vitest_argv, Word};
use super::options::NameFilters;
use std::collections::BTreeMap;

mod flags;
use flags::{takes_value, MODES};

pub(super) struct Narrowing {
    pub(super) line: usize,
    pub(super) project: String,
    pub(super) argument: String,
    pub(super) name: bool,
    pub(super) flag: bool,
}

pub(super) fn narrowings(script: &str, policies: &BTreeMap<&str, NameFilters>) -> Vec<Narrowing> {
    vitest_argv(script)
        .into_iter()
        .flat_map(|argv| invocation(&argv, policies))
        .collect()
}

fn invocation(argv: &[Word], policies: &BTreeMap<&str, NameFilters>) -> Vec<Narrowing> {
    let parsed = parse(argv);
    let selected: Vec<&str> = parsed
        .projects
        .iter()
        .filter(|project| policies.contains_key(project.as_str()))
        .map(String::as_str)
        .collect();
    if selected.is_empty() {
        return Vec::new();
    }
    let file_project = selected[0];
    let name_project = selected
        .iter()
        .copied()
        .find(|project| policies.get(project) == Some(&NameFilters::Reject));
    let mut hits = Vec::new();
    for positional in &parsed.positionals {
        let file = is_file_filter(&positional.text);
        if file {
            hits.push(hit(positional, file_project, false, false));
        } else if let Some(project) = name_project {
            hits.push(hit(positional, project, true, false));
        }
    }
    if let Some(project) = name_project {
        hits.extend(
            parsed
                .name_flags
                .iter()
                .map(|word| hit(word, project, true, true)),
        );
    }
    hits
}

struct Parsed {
    projects: Vec<String>,
    positionals: Vec<Word>,
    name_flags: Vec<Word>,
}

fn parse(argv: &[Word]) -> Parsed {
    let mut parsed = Parsed {
        projects: Vec::new(),
        positionals: Vec::new(),
        name_flags: Vec::new(),
    };
    let mut index = 0;
    let mut ended = false;
    while index < argv.len() {
        let word = &argv[index];
        if ended {
            parsed.positionals.push(word.clone());
            index += 1;
            continue;
        }
        if word.text == "--" {
            ended = true;
            index += 1;
            continue;
        }
        if word.text.starts_with('-') {
            index = take_flag(&mut parsed, argv, index);
            continue;
        }
        if parsed.positionals.is_empty() && MODES.contains(&word.text.as_str()) {
            index += 1;
            continue;
        }
        parsed.positionals.push(word.clone());
        index += 1;
    }
    parsed
}

fn take_flag(parsed: &mut Parsed, argv: &[Word], index: usize) -> usize {
    let (name, inline) = argv[index]
        .text
        .split_once('=')
        .map_or((argv[index].text.as_str(), None), |(name, value)| {
            (name, Some(value))
        });
    if name == "--project" {
        return store_value(argv, index, inline, |value| parsed.projects.push(value));
    }
    if name == "--testNamePattern" || name == "-t" {
        return store_value(argv, index, inline, |value| {
            if !value.is_empty() {
                parsed.name_flags.push(Word {
                    text: value,
                    line: value_line(argv, index, inline),
                    dynamic: false,
                });
            }
        });
    }
    if takes_value(name) {
        if inline.is_some() {
            return index + 1;
        }
        if consumes_next(argv, index) {
            return index + 2;
        }
    }
    index + 1
}

fn store_value(
    argv: &[Word],
    index: usize,
    inline: Option<&str>,
    store: impl FnOnce(String),
) -> usize {
    if let Some(value) = inline {
        store(value.to_string());
        return index + 1;
    }
    if let Some(next) = argv
        .get(index + 1)
        .filter(|word| !word.text.starts_with('-'))
    {
        store(next.text.clone());
        return index + 2;
    }
    index + 1
}

fn value_line(argv: &[Word], index: usize, inline: Option<&str>) -> usize {
    if inline.is_some() {
        argv[index].line
    } else {
        argv.get(index + 1)
            .map(|word| word.line)
            .unwrap_or(argv[index].line)
    }
}

fn consumes_next(argv: &[Word], index: usize) -> bool {
    argv.get(index + 1)
        .is_some_and(|word| !word.text.starts_with('-'))
}

fn hit(word: &Word, project: &str, name: bool, flag: bool) -> Narrowing {
    Narrowing {
        line: word.line,
        project: project.to_string(),
        argument: word.text.clone(),
        name,
        flag,
    }
}

pub(super) fn is_file_filter(text: &str) -> bool {
    if text.contains(['/', '\\', '*', '?', '[']) {
        return true;
    }
    text.contains('.')
        && matches!(
            text.rsplit('.').next(),
            Some(
                "ts" | "tsx"
                    | "js"
                    | "jsx"
                    | "mjs"
                    | "cjs"
                    | "mts"
                    | "cts"
                    | "vue"
                    | "svelte"
                    | "json"
            )
        )
}

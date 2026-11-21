use super::command::{vitest_argv, Word};
use super::options::NameFilters;
use std::collections::BTreeMap;

mod flags;
mod parse;

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
    let parsed = parse::parse(argv);
    if parsed.blocked {
        return Vec::new();
    }
    let selected: Vec<&str> = if parsed.projects.is_empty() {
        policies.keys().copied().collect()
    } else {
        parsed
            .projects
            .iter()
            .filter(|project| policies.contains_key(project.as_str()))
            .map(String::as_str)
            .collect()
    };
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

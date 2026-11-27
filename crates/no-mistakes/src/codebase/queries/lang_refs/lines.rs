use crate::codebase::lang_frontends::{mask_strings, strip_comments_keep_strings};
use crate::codebase::ts_source::SourceStore;
use std::path::Path;

pub(super) fn first_name_line(sources: &SourceStore, path: &Path, name: &str) -> Option<u32> {
    let source = sources.read_path(path).ok()?;
    let masked = mask_strings(&strip_comments_keep_strings(&source));
    let needles = needles(name);
    for (index, line) in masked.lines().enumerate() {
        if needles.iter().any(|needle| contains_word(line, needle)) {
            return Some((index as u32) + 1);
        }
    }
    None
}

pub(super) fn first_import_line(
    sources: &SourceStore,
    path: &Path,
    specifier: &str,
) -> Option<u32> {
    first_name_line(sources, path, specifier).or_else(|| {
        specifier
            .rsplit(['/', '.'])
            .next()
            .filter(|segment| *segment != specifier)
            .and_then(|segment| first_name_line(sources, path, segment))
    })
}

fn needles(name: &str) -> Vec<String> {
    let mut values = vec![name.to_string()];
    if name.contains('.') {
        values.push(name.replace('.', "\\"));
        values.push(name.replace('.', "::"));
    }
    if let Some(segment) = name.rsplit(['.', '\\', ':']).find(|part| !part.is_empty()) {
        if segment != name {
            values.push(segment.to_string());
        }
    }
    values
}

fn contains_word(line: &str, word: &str) -> bool {
    if word.is_empty() {
        return false;
    }
    line.match_indices(word).any(|(at, _)| {
        let before = line[..at].chars().next_back();
        let after = line[at + word.len()..].chars().next();
        !before.is_some_and(is_ident) && !after.is_some_and(is_ident)
    })
}

fn is_ident(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

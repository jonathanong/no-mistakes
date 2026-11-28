use crate::codebase::lang_frontends::{mask_strings, strip_comments_keep_strings};
use crate::codebase::ts_source::SourceStore;
use std::path::Path;

pub(super) fn first_name_line(sources: &SourceStore, path: &Path, name: &str) -> Option<u32> {
    let source = sources.read_path(path).ok()?;
    let normalized = if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
        std::borrow::Cow::Owned(strip_rust_lifetimes(&source))
    } else {
        std::borrow::Cow::Borrowed(source.as_ref())
    };
    let masked = mask_strings(&strip_comments_keep_strings(&normalized));
    let needles = needles(name);
    for (index, line) in masked.lines().enumerate() {
        if needles.iter().any(|needle| contains_word(line, needle)) {
            return Some((index as u32) + 1);
        }
    }
    None
}

fn strip_rust_lifetimes(source: &str) -> String {
    let chars: Vec<char> = source.chars().collect();
    let mut out = String::with_capacity(source.len());
    for (index, &ch) in chars.iter().enumerate() {
        if ch == '\'' && is_rust_lifetime(&chars, index) {
            out.push(' ');
        } else {
            out.push(ch);
        }
    }
    out
}

fn is_rust_lifetime(chars: &[char], index: usize) -> bool {
    if !chars
        .get(index + 1)
        .is_some_and(|ch| ch.is_ascii_alphabetic() || *ch == '_')
    {
        return false;
    }
    let mut end = index + 2;
    while chars
        .get(end)
        .is_some_and(|ch| ch.is_ascii_alphanumeric() || *ch == '_')
    {
        end += 1;
    }
    chars.get(end) != Some(&'\'')
}

pub(super) fn first_import_line(
    sources: &SourceStore,
    path: &Path,
    specifier: &str,
) -> Option<u32> {
    literal_line(sources, path, specifier).or_else(|| {
        specifier
            .rsplit(['/', '.'])
            .next()
            .filter(|segment| *segment != specifier && !segment.is_empty())
            .and_then(|segment| first_name_line(sources, path, segment))
    })
}

fn literal_line(sources: &SourceStore, path: &Path, specifier: &str) -> Option<u32> {
    if specifier.is_empty() {
        return None;
    }
    let source = sources.read_path(path).ok()?;
    let text = strip_comments_keep_strings(&source);
    let masked = mask_strings(&text);
    for (index, (line, masked_line)) in text.lines().zip(masked.lines()).enumerate() {
        if has_exact_quoted(line, specifier) || contains_word(masked_line, specifier) {
            return Some((index as u32) + 1);
        }
    }
    None
}

fn has_exact_quoted(line: &str, specifier: &str) -> bool {
    ['"', '\'', '`'].iter().any(|quote| {
        let wrapped = format!("{quote}{specifier}{quote}");
        line.contains(&wrapped)
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
    ch.is_alphanumeric() || ch == '_'
}

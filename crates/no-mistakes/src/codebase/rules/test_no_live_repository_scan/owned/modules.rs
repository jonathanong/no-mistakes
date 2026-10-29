use std::path::Path;

pub(super) fn specifier_stem(specifier: &str) -> Option<String> {
    let mut segments = specifier.rsplit('/').filter(|segment| !segment.is_empty());
    let mut last = segments.next()?;
    if strip_extension(last) == "index" {
        last = segments.next()?;
    }
    let stem = strip_extension(last);
    (!matches!(stem, "" | "." | "..")).then(|| stem.to_string())
}

pub(super) fn file_stem(file: &Path) -> Option<String> {
    let stem = file.file_stem()?.to_str()?;
    if stem == "index" {
        return file.parent()?.file_name()?.to_str().map(str::to_string);
    }
    Some(stem.to_string())
}

fn strip_extension(segment: &str) -> &str {
    match segment.rsplit_once('.') {
        Some((stem, "js" | "jsx" | "ts" | "tsx" | "mjs" | "cjs" | "mts" | "cts" | "json")) => stem,
        _ => segment,
    }
}

#[cfg(test)]
#[path = "modules_tests.rs"]
mod tests;

use super::super::BaseDir;
use super::types::Extracted;
use regex::Regex;

pub(crate) fn workspace_filters(source: &str) -> Vec<Extracted> {
    let filter = Regex::new(r#"--filter\s+(?:\\\s*)?(?:"([^"]+)"|'([^']+)'|([^\s'"]+))"#)
        .expect("pnpm filter pattern is valid");
    let mut extracted = Vec::new();
    let mut index = 0;
    for capture in filter.captures_iter(source) {
        let raw = capture
            .get(1)
            .or_else(|| capture.get(2))
            .or_else(|| capture.get(3))
            .map(|match_| match_.as_str())
            .unwrap_or_default();
        let Some(path) = normalize_filter(raw) else {
            continue;
        };
        if is_guarded(
            command_scope(source, capture.get(0).unwrap().start()),
            &path,
        ) || path.starts_with('!')
        {
            continue;
        }
        extracted.push(Extracted {
            field: format!("pnpm filter {index}"),
            allow_globs: path.contains('*') || path.contains('?') || path.contains('{'),
            base_dir: BaseDir::Root,
            value: path,
        });
        index += 1;
    }
    extracted
}

fn command_scope(source: &str, offset: usize) -> &str {
    let line_start = source[..offset].rfind('\n').map_or(0, |index| index + 1);
    let mut line_end = source[offset..]
        .find('\n')
        .map_or(source.len(), |index| offset + index);
    while source[line_start..line_end].trim_end().ends_with('\\') {
        let next_start = line_end.saturating_add(1);
        let Some(next_end) = source[next_start..]
            .find('\n')
            .map(|index| next_start + index)
        else {
            line_end = source.len();
            break;
        };
        line_end = next_end;
    }
    &source[line_start..line_end]
}

fn normalize_filter(raw: &str) -> Option<String> {
    let mut path = raw.trim().trim_end_matches(';');
    if let Some(braced) = path.strip_prefix('{') {
        path = braced.strip_suffix("}...").unwrap_or(braced);
        path = path.strip_suffix('}').unwrap_or(path);
    } else {
        path = path.strip_suffix("...").unwrap_or(path);
    }
    if !path.starts_with("./") || path == "./" {
        return None;
    }
    Some(path.to_string())
}

fn is_guarded(source: &str, path: &str) -> bool {
    if path.contains('*') || path.contains('?') || path.contains('{') {
        return false;
    }
    let normalized = path.trim_start_matches("./");
    let escaped = regex::escape(normalized);
    let directory = format!(r#"(?:\./)?{escaped}"#);
    let file = format!(r#"(?:\./)?{escaped}(?:/package\.json)?"#);
    let patterns = [
        format!(r#"\[\s+-f\s+[\"']?{file}[\"']?\s*\]"#),
        format!(r#"\[\s+-d\s+[\"']?{directory}[\"']?\s*\]"#),
        format!(r#"test\s+-f\s+[\"']?{file}[\"']?"#),
        format!(r#"test\s+-d\s+[\"']?{directory}[\"']?"#),
    ];
    patterns.iter().any(|pattern| {
        Regex::new(pattern)
            .expect("pnpm filter guard pattern is valid")
            .is_match(source)
    })
}

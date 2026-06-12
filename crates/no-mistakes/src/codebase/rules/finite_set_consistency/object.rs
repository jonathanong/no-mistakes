use regex::Regex;
use std::collections::BTreeSet;

pub(super) fn const_object_body(source: &str, target: &str) -> Option<String> {
    let pattern = format!(r#"\bconst\s+{}\b\s*(?::[^=]+)?="#, regex::escape(target));
    let mat = Regex::new(&pattern).ok()?.find(source)?;
    let open = source[mat.end()..].find('{')? + mat.end();
    let close = matching_brace(source, open)?;
    source.get(open + 1..close).map(str::to_string)
}

pub(super) fn matching_brace(source: &str, open: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut quote = None;
    let mut escaped = false;
    for (idx, ch) in source.char_indices().skip_while(|(idx, _)| *idx < open) {
        if let Some(active_quote) = quote {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == active_quote {
                quote = None;
            }
            continue;
        }
        match ch {
            '"' | '\'' | '`' => quote = Some(ch),
            '{' => depth += 1,
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(idx);
                }
            }
            _ => {}
        }
    }
    None
}

pub(super) fn top_level_object_keys(body: &str) -> BTreeSet<String> {
    top_level_entries(body)
        .into_iter()
        .map(|(key, _)| key)
        .collect()
}

pub(super) fn top_level_property_values(body: &str, property: &str) -> BTreeSet<String> {
    let pattern = format!(
        r#"(?m)(?:^|[,\{{]\s*)(?:"{}"|'{}'|{})\s*:\s*(?:"([^"]+)"|'([^']+)')"#,
        regex::escape(property),
        regex::escape(property),
        regex::escape(property)
    );
    let regex = Regex::new(&pattern).expect("object property regex compiles");
    top_level_entries(body)
        .into_iter()
        .flat_map(|(_, value)| {
            regex
                .captures_iter(&value)
                .filter_map(|captures| {
                    captures
                        .get(1)
                        .or_else(|| captures.get(2))
                        .map(|capture| capture.as_str().to_string())
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn top_level_entries(body: &str) -> Vec<(String, String)> {
    let mut entries = Vec::new();
    let mut rest = body;
    while let Some((key, after_key)) = take_key(rest) {
        let Some(colon) = after_key.find(':') else {
            break;
        };
        let value = &after_key[colon + 1..];
        let end = top_level_value_end(value);
        entries.push((key, value[..end].to_string()));
        rest = trim_ignorable(&value[end..]);
    }
    entries
}

fn take_key(source: &str) -> Option<(String, &str)> {
    let source = trim_ignorable(source);
    let mut chars = source.char_indices();
    let (_, first) = chars.next()?;
    if first == '"' || first == '\'' {
        let end = source[1..].find(first)? + 1;
        return Some((source[1..end].to_string(), &source[end + 1..]));
    }
    let end = source
        .char_indices()
        .find(|(_, ch)| !(ch.is_alphanumeric() || *ch == '_' || *ch == '$' || *ch == '-'))
        .map(|(idx, _)| idx)?;
    Some((source[..end].to_string(), &source[end..]))
}

fn trim_ignorable(mut source: &str) -> &str {
    loop {
        let trimmed = source.trim_start_matches(|ch: char| ch == ',' || ch.is_whitespace());
        if let Some(rest) = trimmed.strip_prefix("//") {
            source = rest.split_once('\n').map_or("", |(_, after)| after);
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("/*") {
            let Some((_, after)) = rest.split_once("*/") else {
                return "";
            };
            source = after;
            continue;
        }
        return trimmed;
    }
}

fn top_level_value_end(source: &str) -> usize {
    let mut depth = 0usize;
    let mut quote = None;
    let mut escaped = false;
    for (idx, ch) in source.char_indices() {
        if let Some(active_quote) = quote {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == active_quote {
                quote = None;
            }
            continue;
        }
        match ch {
            '"' | '\'' | '`' => quote = Some(ch),
            '{' | '[' | '(' => depth += 1,
            '}' | ']' | ')' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => return idx,
            _ => {}
        }
    }
    source.len()
}

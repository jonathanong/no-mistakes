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
    let mut iter = source
        .char_indices()
        .skip_while(|(idx, _)| *idx < open)
        .peekable();
    while let Some((idx, ch)) = iter.next() {
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
        if ch == '/' {
            match iter.peek().copied() {
                Some((_, '/')) => {
                    iter.next();
                    for (_, comment_ch) in iter.by_ref() {
                        if comment_ch == '\n' {
                            break;
                        }
                    }
                    continue;
                }
                Some((_, '*')) => {
                    iter.next();
                    let mut previous = '\0';
                    for (_, comment_ch) in iter.by_ref() {
                        if previous == '*' && comment_ch == '/' {
                            break;
                        }
                        previous = comment_ch;
                    }
                    continue;
                }
                _ => {}
            }
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
    top_level_entries(body)
        .into_iter()
        .filter_map(|(_, value)| direct_object_body(&value).map(|body| top_level_entries(&body)))
        .flatten()
        .filter_map(|(key, value)| {
            (key == property)
                .then(|| quoted_string_literal(value.trim()))
                .flatten()
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
        if let Some(rest) = trimmed.strip_prefix("...") {
            let Some((_, after)) = rest.split_once(',') else {
                return "";
            };
            source = after;
            continue;
        }
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

fn direct_object_body(value: &str) -> Option<String> {
    let value = value.trim();
    if !value.starts_with('{') {
        return None;
    }
    let close = matching_brace(value, 0)?;
    value.get(1..close).map(str::to_string)
}

fn quoted_string_literal(value: &str) -> Option<String> {
    let quote = value.chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let end = value[1..].find(quote)? + 1;
    Some(value[1..end].to_string())
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

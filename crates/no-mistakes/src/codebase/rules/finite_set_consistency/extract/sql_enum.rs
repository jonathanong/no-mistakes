use super::*;

pub(in super::super) fn extract_sql_enum(source: &str, target: &str) -> BTreeSet<String> {
    let source = strip_sql_comments(source);
    let pattern = format!(
        r#"(?is)CREATE\s+TYPE\s+{}\s+AS\s+ENUM\s*\("#,
        regex::escape(target)
    );
    Regex::new(&pattern)
        .ok()
        .and_then(|regex| regex.find(&source))
        .and_then(|mat| sql_enum_body(&source[mat.end()..]))
        .map(quoted_strings_sql)
        .unwrap_or_default()
}

fn sql_enum_body(source: &str) -> Option<&str> {
    let mut quote = false;
    let mut chars = source.char_indices().peekable();
    while let Some((idx, ch)) = chars.next() {
        if quote {
            if ch == '\'' {
                if chars.peek().is_some_and(|(_, next)| *next == '\'') {
                    chars.next();
                } else {
                    quote = false;
                }
            }
            continue;
        }
        if ch == '\'' {
            quote = true;
            continue;
        }
        if ch == ')' {
            return Some(&source[..idx]);
        }
    }
    None
}

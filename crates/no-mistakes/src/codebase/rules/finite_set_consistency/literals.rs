use std::collections::BTreeSet;

pub(super) fn quoted_strings_ts(source: &str) -> BTreeSet<String> {
    quoted_strings(source, true)
}

pub(super) fn quoted_strings_sql(source: &str) -> BTreeSet<String> {
    quoted_strings(source, false)
}

fn quoted_strings(source: &str, ts_single_quote_escapes: bool) -> BTreeSet<String> {
    let mut values = BTreeSet::new();
    let mut chars = source.char_indices().peekable();
    while let Some((_, ch)) = chars.next() {
        if ch != '"' && ch != '\'' && !(ts_single_quote_escapes && ch == '`') {
            continue;
        }
        let quote = ch;
        let mut value = String::new();
        let mut escaped = false;
        while let Some((_, literal_ch)) = chars.next() {
            if escaped {
                value.push(literal_ch);
                escaped = false;
                continue;
            }
            if (quote == '"' || (quote == '\'' && ts_single_quote_escapes) || quote == '`')
                && literal_ch == '\\'
            {
                escaped = true;
                continue;
            }
            if literal_ch == quote {
                if quote == '\''
                    && !ts_single_quote_escapes
                    && chars.peek().is_some_and(|(_, next)| *next == '\'')
                {
                    chars.next();
                    value.push('\'');
                    continue;
                }
                values.insert(value);
                break;
            }
            value.push(literal_ch);
        }
    }
    values
}

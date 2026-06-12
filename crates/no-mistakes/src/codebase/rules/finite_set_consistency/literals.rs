use std::collections::BTreeSet;

pub(super) fn quoted_strings(source: &str) -> BTreeSet<String> {
    let mut values = BTreeSet::new();
    let mut chars = source.char_indices().peekable();
    while let Some((_, ch)) = chars.next() {
        if ch != '"' && ch != '\'' {
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
            if quote == '"' && literal_ch == '\\' {
                escaped = true;
                continue;
            }
            if literal_ch == quote {
                if quote == '\'' && chars.peek().is_some_and(|(_, next)| *next == '\'') {
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

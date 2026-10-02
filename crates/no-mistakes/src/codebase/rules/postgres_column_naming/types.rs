pub(super) fn lists_type(types: &[String], data_type: &str) -> bool {
    let actual = base_type(data_type);
    types
        .iter()
        .any(|ty| base_type(ty).eq_ignore_ascii_case(&actual))
}

fn base_type(data_type: &str) -> String {
    let mut out = String::new();
    let mut rest = data_type;
    while let Some(start) = rest.find('(') {
        let after = &rest[start + 1..];
        let Some(end) = after.find(')') else {
            out.push_str(rest);
            return collapse(&out);
        };
        let inner = &after[..end];
        if is_typmod(inner) {
            out.push_str(&rest[..start]);
            rest = &after[end + 1..];
        } else {
            out.push_str(&rest[..=start]);
            rest = after;
        }
    }
    out.push_str(rest);
    collapse(&out)
}

fn is_typmod(inner: &str) -> bool {
    !inner.is_empty()
        && inner
            .chars()
            .all(|ch| ch.is_ascii_digit() || matches!(ch, ',' | '-' | ' ' | '\t'))
}

fn collapse(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

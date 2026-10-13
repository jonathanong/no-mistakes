pub(super) fn mapping_key_len(text: &str, wanted: &str) -> Option<usize> {
    let rest = text.strip_prefix(wanted).or_else(|| {
        ['"', '\''].into_iter().find_map(|quote| {
            text.strip_prefix(quote)?
                .strip_prefix(wanted)?
                .strip_prefix(quote)
        })
    })?;
    let spaces = rest.len() - rest.trim_start().len();
    rest.trim_start()
        .starts_with(':')
        .then_some(text.len() - rest.len() + spaces + 1)
}

pub(super) fn key_boundary(previous: Option<char>) -> bool {
    previous.is_none_or(|char| char.is_whitespace() || char == '{' || char == ',')
}

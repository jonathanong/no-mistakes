pub(super) fn normalize_identifier(identifier: &str) -> String {
    identifier
        .rsplit('.')
        .next()
        .unwrap_or(identifier)
        .trim_matches('"')
        .to_ascii_lowercase()
}

pub(super) fn normalize_table_name(table: &str) -> String {
    table
        .split('.')
        .map(normalize_identifier)
        .collect::<Vec<_>>()
        .join(".")
}

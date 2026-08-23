use super::char_scan::embedded_expressions;
use super::reference_scan::static_references_case_insensitive;

/// Extract static accesses to `context` case-insensitively from embedded
/// GitHub expressions. `allow_bare` is used for `if:` fields, where `${{ }}`
/// is optional.
pub fn static_context_references(
    value: Option<&str>,
    context: &str,
    allow_bare: bool,
) -> Vec<String> {
    let Some(value) = value else {
        return Vec::new();
    };
    if allow_bare {
        return static_references_case_insensitive(Some(value), context);
    }
    let chars: Vec<char> = value.chars().collect();
    let mut references = std::collections::BTreeMap::new();
    for expression in embedded_expressions(&chars) {
        let expression: String = expression.into_iter().collect();
        for reference in static_references_case_insensitive(Some(&expression), context) {
            references
                .entry(reference.to_ascii_lowercase())
                .or_insert(reference);
        }
    }
    references.into_values().collect()
}

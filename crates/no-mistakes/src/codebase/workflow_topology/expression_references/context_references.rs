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
    embedded_expressions(&chars)
        .into_iter()
        .flat_map(|expression| {
            let expression: String = expression.into_iter().collect();
            static_references_case_insensitive(Some(&expression), context)
        })
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect()
}

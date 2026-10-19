use super::EmbeddedSqlFileFacts;

/// Keep a span unless every rule that shares this profile suppresses its line.
///
/// An empty rule list is a direct extract, not a vacuous suppression.
pub(crate) fn keep_unproven_span(source: &str, span: (u32, u32), rule_ids: &[String]) -> bool {
    if rule_ids.is_empty() {
        return true;
    }
    let line = crate::codebase::ts_source::byte_offset_to_line(source, span.0 as usize);
    !rule_ids.iter().all(|rule_id| {
        crate::codebase::ts_source::matching_disable_directive(source, Some(line), rule_id)
            .is_some()
    })
}

pub(crate) fn retain_unproven_spans(facts: &mut EmbeddedSqlFileFacts, source: &str) {
    let rule_ids = facts.postgres_rule_ids.clone();
    facts
        .unproven_member_query_spans
        .retain(|span| keep_unproven_span(source, *span, &rule_ids));
}

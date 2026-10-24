pub(crate) fn recovery_failures(
    parsed: &[super::super::parse::LocatedStatement],
) -> Vec<super::super::types::SqlUnanalyzableStatement> {
    parsed
        .iter()
        .filter_map(|item| item.recovery_failure.clone())
        .collect()
}

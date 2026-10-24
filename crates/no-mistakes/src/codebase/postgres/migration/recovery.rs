pub(crate) fn recovery_failures(
    parsed: &[super::super::parse::LocatedStatement],
) -> Vec<super::super::types::SqlUnanalyzableStatement> {
    let mut failures: Vec<_> = parsed
        .iter()
        .filter_map(|item| item.recovery_failure.clone())
        .collect();
    failures.sort_by_key(|failure| (failure.line, failure.column));
    failures
}

#[cfg(test)]
mod tests;

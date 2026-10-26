use super::*;

/// A fixed spread can contribute the bound receiver and later parameters.
pub(in super::super::super) fn entries(
    collector: &ImportCollector,
    value: &ValueExpression,
    before: u32,
) -> Option<Vec<ValueExpression>> {
    let projection = super::entries(collector, value, before)?;
    if projection.tail.is_some()
        || !projection
            .entries
            .iter()
            .enumerate()
            .all(|(slot, (at, _, possible))| *at == slot && !possible)
    {
        return None;
    }
    Some(
        projection
            .entries
            .into_iter()
            .map(|(_, value, _)| value)
            .collect(),
    )
}

use super::Catalog;
use crate::codebase::postgres::statement_facts::{SqlTriggerEvent, SqlTriggerFact};

pub(super) fn written_columns(trigger: &SqlTriggerFact, catalog: &Catalog<'_>) -> Vec<String> {
    if let Some((_, columns)) = catalog
        .trigger_writes
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case(&trigger.function))
    {
        return columns.clone();
    }
    trigger
        .events
        .iter()
        .filter_map(|event| match event {
            SqlTriggerEvent::Update { columns } if !columns.is_empty() => Some(columns.clone()),
            _ => None,
        })
        .flatten()
        .collect()
}

use super::{Evaluation, Offender};
use crate::codebase::postgres::{
    SchemaCatalog, SqlBoundInputMode, SqlBoundItemKind, SqlBoundQuery,
};
use std::collections::HashMap;

/// Row caps suppress streaming reads, while blocking set inputs remain mandatory.
pub(super) fn collect(
    query: &SqlBoundQuery,
    nested: &[Option<Evaluation>],
    pin_subqueries: &[Vec<Option<Evaluation>>],
    bounded: &[bool],
    catalog: &SchemaCatalog,
) -> Vec<Offender> {
    let mut found = Vec::new();
    for (index, item) in query.items.iter().enumerate() {
        for inner in pin_subqueries[index]
            .iter()
            .flatten()
            .chain(nested[index].iter())
        {
            found.extend(
                inner
                    .offenders
                    .iter()
                    .filter(|offender| !query.capped || offender.blocking_input)
                    .cloned(),
            );
        }
        if !query.capped && !bounded[index] && nested[index].is_none() {
            if let SqlBoundItemKind::Table(name) = &item.kind {
                found.extend(table_offender(name, item.line, catalog));
            }
        }
    }
    let mut positions: HashMap<(String, usize), usize> = HashMap::new();
    let mut out: Vec<Offender> = Vec::new();
    for mut offender in found {
        offender.blocking_input |= query.input_mode == SqlBoundInputMode::Blocking;
        let key = (offender.table.clone(), offender.line);
        if let Some(index) = positions.get(&key) {
            // The same CTE read may occur on streaming and blocking paths: retain either proof.
            out[*index].blocking_input |= offender.blocking_input;
        } else {
            positions.insert(key, out.len());
            out.push(offender);
        }
    }
    out
}

pub(super) fn table_offender(name: &str, line: usize, catalog: &SchemaCatalog) -> Option<Offender> {
    Some(Offender {
        table: catalog.relation(name)?.name.clone(),
        line,
        blocking_input: false,
    })
}

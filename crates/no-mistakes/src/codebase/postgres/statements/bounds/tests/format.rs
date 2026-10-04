use super::{query, SqlBoundItem, SqlBoundItemKind, SqlPinSource};

pub(super) fn item(item: &SqlBoundItem) -> String {
    let pins: Vec<String> = item
        .pins
        .iter()
        // Catalog-resolved qualified reads remain deferred facts in these syntax-only snapshots.
        .filter(|pin| pin.qualified_reads.is_empty())
        .map(|pin| {
            // `~=` is the null-safe `IS NOT DISTINCT FROM`.
            let operator = if pin.null_safe { "~=" } else { "=" };
            match &pin.source {
                SqlPinSource::Value => format!("{}{operator}value", pin.column),
                SqlPinSource::StoredArray(items) => {
                    format!("{}{operator}stored-array#{items:?}", pin.column)
                }
                SqlPinSource::Items(items) => {
                    let items: Vec<String> = items.iter().map(usize::to_string).collect();
                    format!("{}{operator}#{}", pin.column, items.join(","))
                }
                SqlPinSource::Array {
                    items,
                    scalar_columns,
                    indexed_columns: _,
                    cast_types: _,
                } => {
                    format!("{}{operator}array#{items:?}:{scalar_columns:?}", pin.column)
                }
                SqlPinSource::Query(bound) => {
                    format!("{}{operator}({})", pin.column, query(bound))
                }
                // This formatter describes key-credit proof; raw fixtures test read metadata.
                SqlPinSource::ReadQuery(_) => String::new(),
            }
        })
        .filter(|pin| !pin.is_empty())
        .collect();
    let pins = if pins.is_empty() {
        String::new()
    } else {
        format!("[{}]", pins.join(" "))
    };
    match &item.kind {
        SqlBoundItemKind::Table(table) => format!("{table}{pins}"),
        SqlBoundItemKind::Query(bound) => format!("({}){pins}", query(bound)),
        SqlBoundItemKind::Other => format!("other{pins}"),
        SqlBoundItemKind::Opaque => format!("opaque{pins}"),
    }
}

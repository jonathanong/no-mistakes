//! Finite row domains for one unique-key column.
use super::domain_range::{self, Literal, Span};
use crate::codebase::postgres::statements::SqlBoundItem;
use crate::codebase::postgres::{CatalogTable, SchemaCatalog};

pub(super) struct Spec {
    pub(super) columns: Vec<(String, String)>,
    pub(super) eligible: bool,
}

/// Per predicate arm, per unique key, per column.
pub(super) type StatementWidths = Vec<Vec<Vec<Option<u64>>>>;

pub(super) fn spec(
    table: &CatalogTable,
    aliases: &[String],
    physicals: Vec<String>,
    eligible: bool,
) -> Option<Spec> {
    let columns = physicals
        .into_iter()
        .map(|physical| {
            let visible =
                super::super::super::arrays::key_visible(table, aliases, &physical)?.to_owned();
            Some((visible, physical))
        })
        .collect::<Option<_>>()?;
    Some(Spec { columns, eligible })
}

pub(super) fn column_domain(
    table: &CatalogTable,
    catalog: &SchemaCatalog,
    item: &SqlBoundItem,
    (visible, physical): (&str, &str),
    alternative: usize,
) -> Option<u64> {
    let column = table
        .columns
        .iter()
        .find(|column| column.name == physical)?;
    if column.nullable {
        return None;
    }
    let data_type = column.data_type.trim();
    let lower = data_type.to_ascii_lowercase();
    if lower == "boolean" || lower == "bool" {
        return Some(2);
    }
    if let Some(enum_type) = catalog.enum_type(data_type) {
        let count = u64::try_from(enum_type.values.len()).ok()?;
        return (count > 0).then_some(count);
    }
    if domain_range::integer_type(&lower) {
        return integer_domain(table, catalog, item, (visible, physical), alternative);
    }
    if lower == "date" {
        return date_domain(item, visible, alternative);
    }
    None
}

pub(super) fn widths_for(
    table: &CatalogTable,
    catalog: &SchemaCatalog,
    item: &SqlBoundItem,
    alternative: usize,
    columns: &[(String, String)],
    eligible: bool,
) -> Vec<Option<u64>> {
    columns
        .iter()
        .map(|(visible, physical)| {
            eligible
                .then(|| column_domain(table, catalog, item, (visible, physical), alternative))
                .flatten()
        })
        .collect()
}

fn integer_domain(
    table: &CatalogTable,
    catalog: &SchemaCatalog,
    item: &SqlBoundItem,
    (visible, physical): (&str, &str),
    alternative: usize,
) -> Option<u64> {
    let qualifiers = check_qualifiers(catalog, table);
    let mut span = None;
    let mut saw = false;
    for check in &table.check_constraints {
        if !check.validated {
            continue;
        }
        let Some(next) = domain_range::owned_check(&check.definition, physical, &qualifiers) else {
            continue;
        };
        saw = true;
        span = domain_range::intersect(span, Some(next));
        span?;
    }
    if let Some(next) = arm_span(item, visible, alternative, Literal::Integer) {
        saw = true;
        span = domain_range::intersect(span, Some(next));
    }
    saw.then_some(span?.width()).flatten()
}

fn date_domain(item: &SqlBoundItem, visible: &str, alternative: usize) -> Option<u64> {
    arm_span(item, visible, alternative, Literal::Date)?.width()
}

fn check_qualifiers(catalog: &SchemaCatalog, table: &CatalogTable) -> Vec<String> {
    let parts = crate::codebase::postgres::decoded_parts(&table.name);
    let bare = parts.last().cloned().unwrap_or_default();
    let mut names = vec![bare.clone()];
    if parts.len() > 1 {
        names.push(parts.join("."));
    } else if let Some(schema) = catalog.declared_schema() {
        names.push(format!("{schema}.{bare}"));
    }
    names
}

fn arm_span(item: &SqlBoundItem, visible: &str, alternative: usize, kind: Literal) -> Option<Span> {
    let arms = domain_range::query_spans(&item.predicate_conjuncts, item.pin_alternatives);
    let arm = arms.get(alternative).or_else(|| arms.first())?;
    domain_range::closed_span(visible, arm, kind)
}

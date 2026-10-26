use super::rebase::rebase_embedded_lines;
use crate::codebase::postgres::embedded::{EmbeddedSqlFileFacts, EmbeddedSqlKind};
use crate::codebase::postgres::statement_facts::{SqlLocationDemand, SqlStatementFileFacts};
use crate::codebase::postgres::statements::{
    extract_sql_conflict_policy_facts, extract_sql_statement_facts_with_json_writes,
    extract_sql_statement_facts_with_location_demand,
};

pub(super) fn embedded_call_facts_with_json_writes(
    file: &EmbeddedSqlFileFacts,
    collect_bounds: bool,
    variants_only: bool,
    demand: SqlLocationDemand,
    collect_json_writes: bool,
) -> Vec<SqlStatementFileFacts> {
    let exact_locations = demand != SqlLocationDemand::None;
    file.calls
        .iter()
        .enumerate()
        .filter(|(_, call)| !variants_only || !call.variants.is_empty())
        .flat_map(|(call_index, call)| {
            let mirrored = call.mirrors_single_variant();
            call.statement_calls()
                .enumerate()
                .map(move |(variant_index, version)| {
                    (
                        version,
                        !call.variants.is_empty(),
                        mirrored,
                        call_index,
                        variant_index,
                    )
                })
        })
        .filter_map(|(call, variant, mirrored, call_index, variant_index)| {
            let sql = call.sql_text.as_deref()?;
            // Ordering rejects opaque tails before consulting prepared statements.
            // Keep all facts for variants and general/bounds consumers.
            let conflict_only =
                demand == SqlLocationDemand::Conflicts && !collect_bounds && !variant;
            if conflict_only
                && (call.kind == EmbeddedSqlKind::Dynamic
                    || !crate::codebase::postgres::conflict::may_have_insert_conflict(sql))
            {
                return None;
            }
            // General consumers and variants retain the complete projection.
            let projection = if variant || !call.sql_source_positions.is_empty() {
                SqlLocationDemand::Full
            } else {
                demand
            };
            let mut facts = if conflict_only {
                extract_sql_conflict_policy_facts(sql, &call.recovered_placeholder_positions)
            } else if collect_json_writes {
                extract_sql_statement_facts_with_json_writes(
                    sql,
                    collect_bounds,
                    Some(&call.recovered_placeholder_positions),
                    projection,
                    true,
                )
            } else {
                extract_sql_statement_facts_with_location_demand(
                    sql,
                    collect_bounds,
                    &call.recovered_placeholder_positions,
                    projection,
                )
            };
            if call.kind == EmbeddedSqlKind::Dynamic && !variant {
                // Retain upstream policy operations, OFFSET syntax, and write targets
                // from opaque prefixes; literal JSON values still require complete SQL.
                let locations = exact_locations
                    .then(|| facts.variant_locations.take())
                    .flatten();
                let parse_failed = exact_locations && facts.parse_failed;
                facts = SqlStatementFileFacts {
                    statement_kinds: facts.statement_kinds,
                    setting_uses: facts.setting_uses,
                    function_calls: facts.function_calls,
                    offset_uses: facts.offset_uses,
                    writes: facts.writes,
                    variant_locations: locations,
                    parse_failed,
                    parse_failures: if parse_failed {
                        facts.parse_failures
                    } else {
                        Vec::new()
                    },
                    ..Default::default()
                };
            }
            facts.path = file.path.clone();
            if let Some(locations) = &mut facts.variant_locations {
                locations.original_call_line = call.line;
                locations.call_index = call_index;
                locations.variant_index = variant_index;
                locations.caller_lines = file
                    .call_spans
                    .get(call_index)
                    .and_then(|span| file.imported_call_lines.get(span))
                    .cloned()
                    .unwrap_or_default();
                locations.append_sites = call
                    .variants
                    .first()
                    .map_or_else(Vec::new, |variant| variant.append_sites.clone());
            }
            rebase_embedded_lines(&mut facts, &call, variant && !mirrored);
            Some(facts)
        })
        .collect()
}

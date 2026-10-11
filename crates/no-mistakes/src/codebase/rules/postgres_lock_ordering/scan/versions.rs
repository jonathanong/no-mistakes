use super::{CompiledOptions, RuleFinding, SchemaCatalog};
use crate::codebase::postgres::EmbeddedSqlCall;

pub(super) fn findings(
    file: &str,
    source: &str,
    original: &EmbeddedSqlCall,
    opts: &CompiledOptions,
    catalog: Option<&SchemaCatalog>,
) -> Vec<RuleFinding> {
    super::findings_for_call_with_catalog(file, source, original, opts, catalog)
}

pub(super) fn prepared_findings(
    file: &str,
    source: &str,
    (call, call_start, comments): (&EmbeddedSqlCall, u32, &[(u32, u32)]),
    statement: &crate::codebase::postgres::SqlStatementFileFacts,
    opts: &CompiledOptions,
    catalog: Option<&SchemaCatalog>,
    dedup: &mut crate::codebase::rules::VariantFindingDedup,
) -> Vec<RuleFinding> {
    use crate::codebase::postgres::statements::SqlFactSite;
    let mut findings = Vec::new();
    let sql = call.sql_text.as_deref().unwrap();
    // A source marker belongs to this invocation; SQL markers belong only to
    // alternatives that contain them, rather than to a sibling source branch.
    if super::super::directive::has_safe_variant_directive(
        source,
        call_start,
        comments,
        sql,
        &opts.safe_directive,
    ) {
        return findings;
    }
    if statement.parse_failed && super::super::directive::contains_for_update(sql) {
        dedup.push(
            &mut findings,
            statement,
            SqlFactSite::Origin,
            Some(source),
            super::finding(
                file,
                call.line,
                super::unparseable_message(file, call.line, &opts.safe_directive),
                super::UNPARSEABLE_TARGET,
            ),
        );
        return findings;
    }
    let locations = statement
        .variant_locations
        .as_ref()
        .expect("prepared variant provenance");
    for (index, lock) in locations.locking.iter().enumerate() {
        for finding in super::lock_findings(file, call, opts, catalog, std::slice::from_ref(lock)) {
            dedup.push(
                &mut findings,
                statement,
                SqlFactSite::Lock(index),
                Some(source),
                finding,
            );
        }
    }
    findings
}

pub(super) fn findings_for_file(
    file: &str,
    source: &str,
    embedded: &crate::codebase::postgres::EmbeddedSqlFileFacts,
    opts: &CompiledOptions,
    catalog: Option<&SchemaCatalog>,
    statements: Option<&[crate::codebase::postgres::SqlStatementFileFacts]>,
    dedup: &mut crate::codebase::rules::VariantFindingDedup,
) -> anyhow::Result<Vec<RuleFinding>> {
    let mut output = Vec::new();
    let statements = statements
        .map(crate::codebase::rules::index_sql_variants)
        .unwrap_or_default();
    for (call_index, original) in embedded.calls.iter().enumerate() {
        if original.variants.is_empty() {
            output.extend(findings(file, source, original, opts, catalog));
            continue;
        }
        for (variant_index, call) in original.statement_calls().enumerate() {
            let statement = statements
                .get(&(call_index, variant_index))
                .copied()
                .ok_or(anyhow::anyhow!("prepared SQL variant facts are missing"))?;
            output.extend(prepared_findings(
                file,
                source,
                (
                    &call,
                    embedded.call_spans[call_index].0,
                    &embedded.source_comment_spans,
                ),
                statement,
                opts,
                catalog,
                dedup,
            ));
        }
    }
    Ok(output)
}

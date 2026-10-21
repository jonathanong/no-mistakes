use super::{offenders, variant_offenders, CompiledOptions, RULE_ID};
use crate::codebase::check_facts::SqlFactContext;
use crate::codebase::postgres::statements::{SqlBoundItemKind, SqlBoundQuery};
use crate::codebase::postgres::{
    project_sql_bounds, record_sql_policy_verdicts, EmbeddedSqlFileFacts, SchemaCatalog,
    SqlBoundKind, SqlBranchJudgment, SqlStatementFileFacts,
};
use crate::codebase::ts_source::relative_slash_path;
use std::path::Path;

pub(super) fn record(
    root: &Path,
    opts: &CompiledOptions,
    embedded: &[&EmbeddedSqlFileFacts],
    catalog: &SchemaCatalog,
    facts: &dyn SqlFactContext,
) -> anyhow::Result<()> {
    let prepared = facts.prepared();
    let checks = opts
        .statements
        .iter()
        .map(|kind| kind_name(*kind).to_string())
        .collect::<Vec<_>>();
    for file in embedded {
        record_sql_policy_verdicts(
            crate::codebase::postgres::SqlPolicyVerdictContext {
                facts,
                embedded: file,
                statements: prepared.postgres_statements(&file.path, Some(&opts.embedded))?,
            },
            RULE_ID,
            &relative_slash_path(root, &file.path),
            &opts.embedded,
            &checks,
            opts.advise_unanalyzable,
            |check, _, statement| judge(check, statement, opts, catalog),
        );
    }
    Ok(())
}

fn judge(
    check: &str,
    statement: &SqlStatementFileFacts,
    opts: &CompiledOptions,
    catalog: &SchemaCatalog,
) -> SqlBranchJudgment {
    let mut unknown_relation = false;
    for bound in project_sql_bounds(statement, catalog)
        .iter()
        .filter(|bound| kind_name(bound.kind) == check)
    {
        if has_unknown_relation(&bound.query, catalog) {
            unknown_relation = true;
        }
        let evaluate = if statement.variant_locations.is_some() {
            variant_offenders
        } else {
            offenders
        };
        if evaluate(bound, catalog)
            .iter()
            .any(|offender| !opts.allow.contains(&format!("table:{}", offender.table)))
        {
            // A later LIMIT or key predicate could bound a partial statement.
            return SqlBranchJudgment::violation(false);
        }
    }
    if unknown_relation {
        SqlBranchJudgment::abstain("unknown-relation")
    } else {
        SqlBranchJudgment::clear()
    }
}

fn has_unknown_relation(query: &SqlBoundQuery, catalog: &SchemaCatalog) -> bool {
    query.items.iter().any(|item| match &item.kind {
        SqlBoundItemKind::Table(name) => catalog.relation(name).is_none(),
        SqlBoundItemKind::Query(inner) => has_unknown_relation(inner, catalog),
        _ => false,
    })
}

#[cfg(test)]
mod tests;

fn kind_name(kind: SqlBoundKind) -> &'static str {
    match kind {
        SqlBoundKind::Select => "select",
        SqlBoundKind::Update => "update",
        SqlBoundKind::Delete => "delete",
    }
}

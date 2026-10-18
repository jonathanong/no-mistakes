use super::{message, CompiledOptions, SchemaCatalog, SqlStarProjectionFact, RULE_ID};
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::postgres::{
    record_sql_policy_verdicts, EmbeddedSqlFileFacts, SqlBranchJudgment, SqlStatementFileFacts,
};
use crate::codebase::ts_source::relative_slash_path;
use std::path::Path;

pub(super) fn record(
    root: &Path,
    opts: &CompiledOptions,
    embedded: &[&EmbeddedSqlFileFacts],
    catalog: &SchemaCatalog,
    facts: &CheckFactMap,
) -> anyhow::Result<()> {
    let mut checks = vec!["select-star".to_string()];
    if opts.check_returning {
        checks.push("returning-star".to_string());
    }
    for file in embedded {
        record_sql_policy_verdicts(
            crate::codebase::postgres::SqlPolicyVerdictContext {
                facts,
                embedded: file,
                statements: facts.postgres_statements(&file.path, Some(&opts.embedded))?,
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
    let stars = match check {
        "select-star" => statement
            .selects
            .iter()
            .flat_map(|select| &select.star_projections)
            .collect::<Vec<_>>(),
        _ => statement.returning_stars.iter().collect(),
    };
    let mut unknown_relation = false;
    for star in stars {
        if exempt(star, opts) {
            continue;
        }
        let listed = opts
            .relations
            .iter()
            .any(|relation| relation.eq_ignore_ascii_case(&star.relation));
        if !listed && catalog.relation(&star.relation).is_none() {
            unknown_relation = true;
            continue;
        }
        let (kind, verb) = if check == "select-star" {
            ("SELECT *", "reads")
        } else {
            ("RETURNING *", "returns")
        };
        if message(kind, verb, star, opts, catalog).is_some() {
            return SqlBranchJudgment::violation(true);
        }
    }
    if unknown_relation {
        SqlBranchJudgment::abstain("unknown-relation")
    } else {
        SqlBranchJudgment::clear()
    }
}

fn exempt(star: &SqlStarProjectionFact, opts: &CompiledOptions) -> bool {
    star.within_function.as_ref().is_some_and(|function| {
        opts.whole_row_functions
            .iter()
            .any(|allowed| allowed.eq_ignore_ascii_case(function))
    })
}

use super::super::{functions, iteration, select_findings, CompiledOptions};
use super::RULE_ID;
use crate::codebase::check_facts::SqlFactContext;
use crate::codebase::postgres::{
    record_sql_policy_verdicts, EmbeddedSqlFileFacts, SqlBranchJudgment, SqlStatementFileFacts,
};
use crate::codebase::ts_source::relative_slash_path;
use std::path::Path;

pub(super) fn record(
    root: &Path,
    opts: &CompiledOptions,
    embedded: &EmbeddedSqlFileFacts,
    facts: &dyn SqlFactContext,
) -> anyhow::Result<()> {
    let prepared = facts.prepared();
    let checks = enabled_shapes(opts);
    record_sql_policy_verdicts(
        crate::codebase::postgres::SqlPolicyVerdictContext {
            facts,
            embedded,
            statements: prepared.postgres_statements(&embedded.path, Some(&opts.embedded))?,
        },
        RULE_ID,
        &relative_slash_path(root, &embedded.path),
        &opts.embedded,
        &checks,
        opts.advise_unanalyzable,
        |check, _, statement| judge(check, statement, opts),
    );
    Ok(())
}

fn enabled_shapes(opts: &CompiledOptions) -> Vec<String> {
    let shapes = &opts.shapes;
    [
        (
            shapes.correlated_exists_set_operation,
            "correlated-exists-set-operation",
        ),
        (shapes.not_in_subquery, "not-in-subquery"),
        (shapes.count_for_existence, "count-for-existence"),
        (shapes.literal_limit, "literal-limit"),
        (shapes.keyset_only_sweep, "keyset-only-sweep"),
        (shapes.banned_function_call, "banned-function-call"),
    ]
    .into_iter()
    .filter(|(enabled, _)| *enabled)
    .map(|(_, name)| name.to_string())
    .collect()
}

fn judge(
    check: &str,
    statement: &SqlStatementFileFacts,
    opts: &CompiledOptions,
) -> SqlBranchJudgment {
    let file = "";
    let hit = match check {
        "banned-function-call" => functions::located_findings(
            file,
            statement,
            &opts.shapes,
            &opts.banned_functions,
            |line| line,
        )
        .into_iter()
        .any(|(_, finding)| finding.target.as_deref() == Some(check)),
        "correlated-exists-set-operation" | "not-in-subquery" | "count-for-existence" => {
            statement.selects.iter().enumerate().any(|(index, select)| {
                select_findings::located_findings(file, select, &opts.shapes, index, |line| line)
                    .into_iter()
                    .any(|(_, finding)| finding.target.as_deref() == Some(check))
            })
        }
        _ => {
            iteration::located_findings(file, statement, &opts.shapes, &opts.iteration, |line| line)
                .into_iter()
                .any(|(_, finding)| finding.target.as_deref() == Some(check))
        }
    };
    if hit {
        SqlBranchJudgment::violation(check != "keyset-only-sweep")
    } else {
        SqlBranchJudgment::clear()
    }
}

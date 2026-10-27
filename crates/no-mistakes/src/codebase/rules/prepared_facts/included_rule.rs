use super::*;
use crate::codebase::check_facts::SqlFactContext;
use rayon::prelude::*;

pub(crate) fn run_included_postgres_rule<C>(
    context: super::super::PreparedRuleContext<'_>,
    rule_id: &str,
    compile: impl Fn(&RuleDef) -> Result<C> + Sync,
    includes: impl Fn(&C, &str) -> bool + Sync,
    scan: impl Fn(
            &Path,
            &C,
            &[PathBuf],
            &SourceStore,
            &dyn SqlFactContext,
        ) -> Result<super::super::PostgresFindings>
        + Sync,
) -> Result<Vec<RuleFinding>> {
    let super::super::PreparedRuleContext {
        root,
        config,
        all_files,
        sources,
        facts,
    } = context;
    let observer = crate::diagnostics::current();
    let all: Result<Vec<super::super::PostgresFindings>> = config
        .rule_applications(rule_id)
        .into_par_iter()
        .enumerate()
        .map(|(index, rule)| {
            crate::diagnostics::with_observer(observer.clone(), || {
                let compiled = compile(rule)?;
                let files = super::super::selected_rule_files(root, config, rule, all_files)?;
                let files: Vec<PathBuf> = files
                    .into_iter()
                    .filter(|path| {
                        includes(
                            &compiled,
                            &crate::codebase::ts_source::relative_slash_path(root, path),
                        )
                    })
                    .collect();
                let view = facts.for_sql_application_with_advice(index, rule.name.as_deref());
                let view = if matches!(
                    rule_id,
                    "postgres-explicit-columns" | "postgres-required-predicates"
                ) && config.effective_rule_analysis(rule)?.effort
                    == crate::config::v2::Effort::Low
                {
                    view.with_low_sql(rule_id)
                } else {
                    view
                };
                scan(
                    root,
                    &compiled,
                    &files,
                    sources,
                    &view as &dyn SqlFactContext,
                )
            })
        })
        .collect();
    let findings: super::super::PostgresFindings = all?.into_iter().flatten().collect();
    Ok(findings.finish())
}

use super::{CheckFactPlan, Path};
use crate::codebase::postgres::{query_annotation, EmbeddedSqlFileFacts, EmbeddedSqlOptions};

pub(super) fn collect(
    root: &Path,
    path: &Path,
    source: &str,
    program: &oxc_ast::ast::Program<'_>,
    plan: &CheckFactPlan,
    embedded: &[(EmbeddedSqlOptions, EmbeddedSqlFileFacts)],
) -> Vec<(
    EmbeddedSqlOptions,
    query_annotation::QueryAnnotationFileFacts,
)> {
    if !plan.query_annotation {
        return Vec::new();
    }
    let options = plan.query_annotation_options.clone().unwrap_or_else(|| {
        embedded
            .iter()
            .map(|(options, _)| options.clone())
            .collect()
    });
    let relative = crate::codebase::ts_source::relative_slash_path(root, path);
    let array_prototype_exposed =
        plan.defer_query_annotation && query_annotation::array_prototype_exposed(program);
    options
        .into_iter()
        .map(|options| {
            let mut facts = if plan.defer_query_annotation {
                query_annotation::QueryAnnotationFileFacts::annotation_scope(
                    array_prototype_exposed,
                )
            } else {
                query_annotation::collect(program, source, &options)
            };
            facts.in_scope = plan.query_annotation_options.as_ref().and_then(|_| {
                plan.embedded_sql_rule_ids.get(&options).map(|scans| {
                    scans
                        .iter()
                        .any(|scan| scan.annotation_seed && scan.paths.includes(&relative))
                })
            });
            if plan.defer_query_annotation {
                facts.syntax_in_scope = plan.annotation_scans().get(&options).map(|scans| {
                    scans
                        .iter()
                        .any(|scan| scan.includes_annotation(path, &relative))
                });
            }
            (options, facts)
        })
        .collect()
}

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
#[path = "program_annotation_tests.rs"]
mod tests;

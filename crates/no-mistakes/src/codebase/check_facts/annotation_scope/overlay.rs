use super::{
    AnalysisSession, BatchCheckFactRequest, CheckFileFacts, CollectedDemand, FileIdMap, Path,
};
use crate::codebase::postgres::query_annotation::{self, project::SyntaxDemand};

type Updates = Vec<(usize, query_annotation::QueryAnnotationFileFacts)>;

pub(super) fn lower<'a>(
    session: &AnalysisSession,
    files: &FileIdMap<CheckFileFacts>,
    demand: &SyntaxDemand,
    tasks: &'a [CollectedDemand],
) -> Vec<(&'a Path, Result<Updates, String>)> {
    if tasks.is_empty() {
        return Vec::new();
    }
    let cache = crate::ast::current_request_parse_cache().expect("annotation program owner");
    tasks
        .iter()
        .filter_map(|(path, _, _)| {
            let profiles = demand.get(path)?;
            let file = files.get(path).expect("demanded annotation file");
            let updates = crate::codebase::fact_panic::isolate(session, path, || {
                crate::diagnostics::hot_files::with_file(path, || {
                    cache
                        .with_existing_program(path, |program, source| {
                            file.query_annotation
                                .iter()
                                .enumerate()
                                .filter(|(_, (options, _))| profiles.contains(options))
                                .map(|(index, (options, placeholder))| {
                                    let mut facts =
                                        query_annotation::collect(program, source, options);
                                    facts.in_scope = placeholder.in_scope;
                                    (index, facts)
                                })
                                .collect()
                        })
                        .expect("successfully parsed annotation program stays on its owner")
                })
            });
            Some((path.as_path(), updates))
        })
        .collect()
}

pub(super) fn install<'a>(
    request: &BatchCheckFactRequest,
    files: &mut FileIdMap<CheckFileFacts>,
    results: impl IntoIterator<Item = (&'a Path, Result<Updates, String>)>,
) -> crate::fx::FxHashMap<std::path::PathBuf, String> {
    let mut failures = crate::fx::fx_map();
    for (path, updates) in results {
        let file = files.get_mut(path).expect("demanded annotation file");
        match updates {
            Ok(updates) => {
                for (index, facts) in updates {
                    file.query_annotation[index].1 = facts;
                }
            }
            Err(error) => {
                *file = super::super::file_parse_error::panic_facts(
                    &request.root,
                    path,
                    &request.plan,
                    error.clone(),
                    file.source.clone().or_else(|| file.ts.source.clone()),
                );
                failures.insert(path.to_path_buf(), error);
            }
        }
    }
    failures
}

pub(super) fn fail_other_variants(
    requests: &[BatchCheckFactRequest],
    mut task: CollectedDemand,
    error: &str,
) -> CollectedDemand {
    let (path, variants, facts) = &mut task;
    for (variant, slot) in variants.iter().zip(facts) {
        if let Some(file) = slot {
            *file = super::super::file_parse_error::panic_facts(
                &requests[variant.request].root,
                path,
                &variant.plan,
                error.to_owned(),
                file.source.clone().or_else(|| file.ts.source.clone()),
            );
        }
    }
    task
}

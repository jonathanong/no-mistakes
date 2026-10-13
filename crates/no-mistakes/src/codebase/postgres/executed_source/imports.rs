use super::{EmbeddedSqlOptions, ExecutedPostgresSource};
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::check_facts::CheckFileFacts;
use crate::codebase::postgres::extract_embedded_sql_from_program;
use crate::codebase::ts_source::facts::{
    collect_file_facts_from_program, TsFactContext, TsFactPlan,
};
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub(super) fn resolve_supplied_relative(
    from: &Path,
    specifier: &str,
    supplied: &crate::fx::PathSet,
) -> Option<PathBuf> {
    let base = crate::codebase::ts_resolver::normalize_path(&from.parent()?.join(specifier));
    let mut candidates = vec![base.clone()];
    if base.extension().is_none() {
        let path = base.to_string_lossy();
        for extension in [".mts", ".ts", ".tsx", ".mjs", ".js", ".jsx"] {
            candidates.push(PathBuf::from(format!("{path}{extension}")));
            candidates.push(base.join(format!("index{extension}")));
        }
    }
    let mut found = candidates
        .into_iter()
        .filter(|path| supplied.contains(path));
    let first = found.next()?;
    found.next().is_none().then_some(first)
}

pub(super) fn extract_source_with_imports(
    source: &ExecutedPostgresSource,
    options: &EmbeddedSqlOptions,
    session: &AnalysisSession,
) -> Result<(PathBuf, CheckFileFacts)> {
    let path = Path::new(&source.file_name);
    session.with_program(
        path,
        &Arc::from(source.source.as_str()),
        |program, source_text| {
            let embedded = extract_embedded_sql_from_program(path, program, source_text, options);
            let mut annotation =
                super::super::query_annotation::collect(program, source_text, options);
            annotation.precise_records = true;
            let ts = collect_file_facts_from_program(
                path,
                TsFactPlan::imports(),
                &TsFactContext::default(),
                source_text,
                program,
                None,
                None,
            );
            (
                path.to_path_buf(),
                CheckFileFacts {
                    ts: Arc::new(ts),
                    embedded_sql: vec![(options.clone(), embedded)],
                    query_annotation: vec![(options.clone(), annotation)],
                    ..Default::default()
                },
            )
        },
    )
}

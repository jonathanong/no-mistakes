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
    if let Some(extension) = base.extension().and_then(|value| value.to_str()) {
        let source_extensions: &[&str] = match extension {
            "js" => &["ts", "tsx"],
            "mjs" => &["mts"],
            "cjs" => &["cts"],
            _ => &[],
        };
        if let Some(source) = source_extensions
            .iter()
            .map(|extension| base.with_extension(extension))
            .find(|path| supplied.contains(path))
        {
            return Some(source);
        }
        if supplied.contains(&base) {
            return Some(base);
        }
        return (extension == "js")
            .then(|| base.with_extension("jsx"))
            .filter(|path| supplied.contains(path));
    }
    let path = base.to_string_lossy();
    if supplied.contains(&base) {
        return Some(base);
    }
    for extension in [".mts", ".ts", ".tsx", ".mjs", ".js", ".jsx", ".cjs", ".cts"] {
        let candidate = PathBuf::from(format!("{path}{extension}"));
        if supplied.contains(&candidate) {
            return Some(candidate);
        }
    }
    for extension in [".mts", ".ts", ".tsx", ".mjs", ".js", ".jsx", ".cjs", ".cts"] {
        let candidate = base.join(format!("index{extension}"));
        if supplied.contains(&candidate) {
            return Some(candidate);
        }
    }
    None
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

#[cfg(test)]
mod tests;

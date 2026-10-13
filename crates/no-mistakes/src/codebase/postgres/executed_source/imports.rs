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
        let mut found = source_extensions
            .iter()
            .map(|extension| base.with_extension(extension))
            .filter(|path| supplied.contains(path));
        if let Some(first) = found.next() {
            return found.next().is_none().then_some(first);
        }
        return supplied.contains(&base).then_some(base);
    }
    let path = base.to_string_lossy();
    let mut candidates = vec![base.clone()];
    for extension in [".mts", ".ts", ".tsx", ".cts", ".mjs", ".js", ".jsx", ".cjs"] {
        candidates.push(PathBuf::from(format!("{path}{extension}")));
        candidates.push(base.join(format!("index{extension}")));
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

#[cfg(test)]
mod tests {
    use super::resolve_supplied_relative;
    use std::path::{Path, PathBuf};

    #[test]
    fn supplied_node_next_imports_prefer_source_and_support_commonjs() {
        let files = [
            "src/helper.ts",
            "src/helper.js",
            "src/common.cts",
            "src/index.cjs",
        ]
        .into_iter()
        .map(PathBuf::from)
        .collect();
        let from = Path::new("src/consumer.mts");
        assert_eq!(
            resolve_supplied_relative(from, "./helper.js", &files),
            Some(PathBuf::from("src/helper.ts"))
        );
        assert_eq!(
            resolve_supplied_relative(from, "./common.cjs", &files),
            Some(PathBuf::from("src/common.cts"))
        );
        assert_eq!(
            resolve_supplied_relative(from, "./common", &files),
            Some(PathBuf::from("src/common.cts"))
        );
    }
}

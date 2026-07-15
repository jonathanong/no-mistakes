use super::CheckFactVariant;
use crate::codebase::check_facts::CheckFileFacts;
use crate::codebase::ts_source::facts::TsFileFacts;
use std::path::Path;
use std::sync::Arc;

pub(super) fn read_errors(
    path: &Path,
    len: usize,
    error: impl std::fmt::Display,
) -> Vec<Option<CheckFileFacts>> {
    let parse_error = format!("failed to read {}: {error}", path.display());
    (0..len)
        .map(|_| {
            Some(CheckFileFacts {
                ts: Arc::new(TsFileFacts {
                    parse_error: Some(parse_error.clone()),
                    ..TsFileFacts::default()
                }),
                parse_error: Some(parse_error.clone()),
                ..CheckFileFacts::default()
            })
        })
        .collect()
}

pub(super) fn fill_parse_errors(
    results: &mut [Option<CheckFileFacts>],
    variants: Vec<(usize, &CheckFactVariant<'_>)>,
    path: &Path,
    source: &Arc<str>,
    legacy: bool,
    error: anyhow::Error,
) {
    for (index, variant) in variants {
        let stored_source = super::should_store_source(variant.plan).then(|| Arc::clone(source));
        let parse_error = if legacy {
            error.to_string()
        } else {
            format!("unsupported file type: {}", path.display())
        };
        results[index] = Some(CheckFileFacts {
            ts: Arc::new(TsFileFacts {
                parse_error: Some(parse_error.clone()),
                source: stored_source.as_deref().map(str::to_owned),
                ..TsFileFacts::default()
            }),
            source: stored_source,
            parse_error: Some(parse_error),
            ..CheckFileFacts::default()
        });
    }
}

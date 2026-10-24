use super::CompiledOptions;
use crate::codebase::check_facts::CheckFactMap;
use std::path::Path;

pub(super) fn proves_missing(
    facts: &CheckFactMap,
    path: &Path,
    opts: &CompiledOptions,
    span: (u32, u32),
) -> bool {
    let Some(file) = facts.ts.get(path) else {
        return false;
    };
    let Some(tag) = file
        .query_annotation_heads
        .as_ref()
        .and_then(|heads| heads.loop_carried_missing.get(&span))
    else {
        return false;
    };
    tag.as_ref().is_none_or(|tag| {
        file.query_annotation.iter().any(|(options, annotation)| {
            options == &opts.embedded && annotation.trusts_loop_tag(tag)
        })
    })
}

pub(super) fn verdict(
    facts: &CheckFactMap,
    path: &Path,
    opts: &CompiledOptions,
    span: (u32, u32),
) -> Option<super::AnnotationVerdict> {
    proves_missing(facts, path, opts, span).then_some((
        crate::codebase::postgres::SqlVerdictStatus::Violation,
        Some(crate::codebase::postgres::SqlEvidenceKind::Exact),
        Vec::new(),
        None,
    ))
}

#[cfg(test)]
#[path = "loop_carried_tests.rs"]
mod tests;

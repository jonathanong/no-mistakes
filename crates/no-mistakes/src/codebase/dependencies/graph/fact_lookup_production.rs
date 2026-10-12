use super::{ExtractedImport, TsFileFacts};
use crate::codebase::check_facts::CheckFactMap;
use std::path::Path;

pub(super) fn check_file_facts_for_path<'a>(
    facts: &'a CheckFactMap,
    path: &Path,
) -> Option<&'a std::sync::Arc<crate::codebase::check_facts::CheckFileFacts>> {
    facts.ts.get(path).or_else(|| {
        let normalized = crate::codebase::ts_resolver::normalize_path(path);
        facts.ts.get(&normalized)
    })
}

pub(super) fn from_ts<'a>(facts: &'a TsFileFacts, path: &Path) -> Option<&'a [ExtractedImport]> {
    if facts.operational_error.is_some() {
        return Some(&[]);
    }
    crate::ast::legacy_symbols_share_standard_parse(path).then_some(facts.imports.as_slice())
}

pub(super) fn from_check<'a>(
    facts: &'a CheckFactMap,
    path: &Path,
) -> Option<&'a [ExtractedImport]> {
    let facts = check_file_facts_for_path(facts, path)?;
    if let Some(imports) = &facts.production_imports {
        return Some(imports.as_slice());
    }
    from_ts(&facts.ts, path)
}

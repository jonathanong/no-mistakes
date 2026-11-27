use super::exports::ExportKey;
use super::{CheckFileFacts, ExportLookup, FileIdMap, FxHashMap, FxHashSet, ImportRef, PathBuf};
use crate::codebase::testing_analysis::reliability::playwright_provenance::WrapperCall;
use std::sync::Arc;

type ImportUse = (PathBuf, ImportRef, Vec<WrapperCall>);
type Unverified = (Option<&'static str>, Vec<(String, Arc<str>)>);

pub(super) fn apply<'a>(
    lookup: &ExportLookup<'_>,
    spec_files: impl Iterator<Item = &'a PathBuf>,
    facts: &mut FileIdMap<CheckFileFacts>,
    verified: &FxHashSet<ImportUse>,
    resolved_map: &FxHashMap<ImportUse, Unverified>,
    uncertainties: &FxHashMap<ExportKey, Option<&'static str>>,
) {
    for path in spec_files {
        let Some(file) = facts.get_mut(path) else {
            continue;
        };
        for location in &mut file.test_reliability {
            location.playwright_candidates.retain(|import| {
                let key = (path.clone(), import.clone(), Vec::new());
                if let Some((_, exports)) = resolved_map.get(&key) {
                    location
                        .resolved_playwright_exports
                        .extend(exports.iter().cloned());
                }
                verified.contains(&key)
                    || lookup.local(path, import).is_none_or(|export| {
                        uncertainties.get(&export).is_some_and(Option::is_some)
                    })
            });
            location.resolved_playwright_exports.sort();
            location.resolved_playwright_exports.dedup();
            let Some(import) = location.playwright_import.clone() else {
                continue;
            };
            let key = (path.clone(), import, location.playwright_wrappers.clone());
            if verified.contains(&key) {
                location.playwright_import = None;
                location.playwright_wrappers.clear();
            } else if let Some((reason, exports)) = resolved_map.get(&key) {
                location.unresolved_reason = *reason;
                location.resolved_playwright_exports = exports.clone();
            }
        }
    }
}

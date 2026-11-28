use super::exports::{ExportKey, UNRESOLVED_EXPORT};
use super::{
    CheckFileFacts, ExportLookup, FileIdMap, FxHashMap, FxHashSet, ImportRef, Path, PathBuf,
};
use crate::codebase::dependencies::graph::EdgeKind;
use crate::codebase::testing_analysis::reliability::playwright_provenance::WrapperCall;
use crate::codebase::ts_source::relative_slash_path;
use crate::edge_index::EdgeIndex;
use std::sync::Arc;

type Candidate = (PathBuf, ImportRef, Vec<WrapperCall>);

struct Unverified {
    reason: Option<&'static str>,
    malformed: Option<String>,
    exports: Vec<(String, Arc<str>)>,
}

pub(super) fn apply<'a>(
    root: &Path,
    facts: &mut FileIdMap<CheckFileFacts>,
    spec_files: impl Iterator<Item = &'a PathBuf>,
    pending: FxHashSet<Candidate>,
    lookup: &ExportLookup<'_>,
    relationships: &EdgeIndex<ExportKey, EdgeKind>,
) {
    let mut proofs = FxHashMap::default();
    let mut verified = FxHashSet::default();
    let mut resolved_map = FxHashMap::default();
    let mut uncertainties = FxHashMap::default();
    let mut malformed_modules = FxHashMap::default();
    for item in pending {
        let (path, import, wrappers) = &item;
        let direct = import.is_playwright_test();
        let base = (!direct).then(|| lookup.local(path, import)).flatten();
        let base_ok = direct
            || base.as_ref().is_some_and(|key| {
                *proofs
                    .entry(key.clone())
                    .or_insert_with(|| lookup.proves_playwright(relationships, key))
            });
        let mut wrappers_ok = true;
        let mut wrapper_reason = None;
        let mut malformed_wrapper = None;
        let mut uncertainty = |key: &ExportKey| {
            *uncertainties
                .entry(key.clone())
                .or_insert_with(|| lookup.uncertainty_reason(relationships, key))
        };
        let mut malformed = |key: &ExportKey| {
            *malformed_modules
                .entry(key.clone())
                .or_insert_with(|| lookup.malformed_fixture(relationships, key))
        };
        for wrapper in wrappers {
            let key = lookup.local(path, &wrapper.callee);
            if key
                .as_ref()
                .is_some_and(|key| lookup.proves_wrapper(key, wrapper.arg_index))
            {
                continue;
            }
            wrappers_ok = false;
            wrapper_reason = wrapper_reason.or_else(|| {
                uncertainty(key.as_ref()?).filter(|reason| *reason != UNRESOLVED_EXPORT)
            });
            malformed_wrapper = malformed_wrapper.or_else(|| malformed(key.as_ref()?));
        }
        if base_ok && wrappers_ok {
            verified.insert(item);
            continue;
        }
        let reason = base
            .as_ref()
            .and_then(&mut uncertainty)
            .filter(|reason| *reason != UNRESOLVED_EXPORT)
            .or(wrapper_reason)
            .or((!wrappers_ok).then_some("fixture wrapper return could not be traced statically"));
        let malformed = base
            .as_ref()
            .filter(|_| !base_ok)
            .and_then(&mut malformed)
            .or(malformed_wrapper)
            .map(|file| relative_slash_path(root, file));
        let exports = base
            .iter()
            .filter(|_| wrappers_ok)
            .flat_map(|key| lookup.reachable_exports(relationships, key))
            .map(|target| (relative_slash_path(root, &target.file), target.name))
            .collect();
        resolved_map.insert(
            item,
            Unverified {
                reason,
                malformed,
                exports,
            },
        );
    }
    for path in spec_files {
        let Some(file) = facts.get_mut(path) else {
            continue;
        };
        for location in &mut file.test_reliability {
            location.playwright_candidates.retain(|import| {
                let key = (path.clone(), import.clone(), Vec::new());
                if let Some(result) = resolved_map.get(&key) {
                    location
                        .resolved_playwright_exports
                        .extend(result.exports.iter().cloned());
                    if location.malformed_playwright_fixture.is_none() {
                        location
                            .malformed_playwright_fixture
                            .clone_from(&result.malformed);
                    }
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
            } else if let Some(result) = resolved_map.get(&key) {
                location.unresolved_reason = result.reason;
                location
                    .resolved_playwright_exports
                    .clone_from(&result.exports);
                location
                    .malformed_playwright_fixture
                    .clone_from(&result.malformed);
            }
        }
    }
}

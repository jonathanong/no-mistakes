use super::{CheckFactPlan, CheckFileFacts, FactRequest};
use crate::codebase::dependencies::extract::is_indexable;
use crate::codebase::rules::test_reliability::playwright_provenance::ImportRef;
use crate::codebase::ts_resolver::{
    ScopedImportResolver, TsConfigCatalog, WorkspaceFallbackResolver,
};
use crate::codebase::ts_source::{relative_slash_path, FileIdMap, SourceStore};
use crate::fx::{FxHashMap, FxHashSet, PathSet};
use std::path::{Path, PathBuf};

mod exports;
use exports::ExportLookup;

pub(super) fn project(
    session: FactRequest<'_>,
    root: &Path,
    sources: &SourceStore,
    plan: &CheckFactPlan,
    facts: &mut FileIdMap<CheckFileFacts>,
) {
    if !plan.playwright_fixture_support {
        return;
    }
    let spec_files = || {
        plan.test_reliability_paths
            .iter()
            .filter_map(|(path, demand)| demand.includes_playwright_tests().then_some(path))
    };
    let pending = spec_files()
        .filter_map(|path| Some((path, facts.get(path)?)))
        .flat_map(|(path, file)| {
            file.test_reliability.iter().flat_map(|location| {
                location
                    .playwright_import
                    .iter()
                    .map(|import| {
                        (
                            path.clone(),
                            import.clone(),
                            location.playwright_wrappers.clone(),
                        )
                    })
                    .chain(
                        location
                            .playwright_candidates
                            .iter()
                            .map(|import| (path.clone(), import.clone(), Vec::new())),
                    )
            })
        })
        .collect::<FxHashSet<_>>();
    if pending.is_empty() {
        return;
    }
    let visible = sources
        .inventory()
        .paths()
        .iter()
        .cloned()
        .collect::<PathSet>();
    let workspace = super::relative_scope::workspace_for(session.session, root, sources);
    let fallback;
    let catalog = match plan.query_annotation_catalog.as_deref() {
        Some(catalog) => catalog,
        None => {
            fallback = TsConfigCatalog::from_visible_and_sources_with_workspace(
                root,
                &[root.to_path_buf()],
                &sources.inventory().paths(),
                sources,
                &workspace,
            );
            &fallback
        }
    };
    let scoped = ScopedImportResolver::new_in_session(catalog, &visible, session.session);
    let resolver = WorkspaceFallbackResolver::new(&scoped, &workspace, &visible);
    let support_plan = CheckFactPlan {
        playwright_fixture_support: true,
        ..CheckFactPlan::default()
    };
    let mut lookup = ExportLookup::new(&resolver, &visible);
    let mut wave = Vec::new();
    for (path, import, wrappers) in &pending {
        if !import.is_playwright_test() {
            if let Some(key) = lookup.local(path, import) {
                wave.push(key);
            }
        }
        for wrapper in wrappers {
            if let Some(key) = lookup.local(path, &wrapper.callee) {
                wave.push(key);
            }
        }
    }
    let mut support = FxHashMap::default();
    let mut attempted = PathSet::default();
    while !wave.is_empty() && crate::invocation::check_timeout().is_ok() {
        wave.sort();
        wave.dedup();
        let mut missing = wave
            .iter()
            .filter(|key| {
                facts
                    .get(&key.file)
                    .is_none_or(|file| !file.parsed && file.parse_error.is_none())
                    && attempted.insert(key.file.clone())
            })
            .map(|key| key.file.clone())
            .collect::<Vec<_>>();
        missing.sort();
        support.extend(super::collect_helpers::collect_fact_entries_with_sources(
            session,
            root,
            &missing,
            &support_plan,
            None,
            sources,
        ));
        let mut next = Vec::new();
        for key in wave {
            if lookup.contains(&key) {
                continue;
            }
            let file = support.get(&key.file).or_else(|| facts.get(&key.file));
            lookup.add(&mut next, key, file);
        }
        wave = next;
    }
    let relationships = lookup.finalize();
    let mut proofs = FxHashMap::default();
    let mut verified = FxHashSet::default();
    let mut resolved_map = FxHashMap::default();
    for item in pending {
        let (path, import, wrappers) = &item;
        let base_ok = if import.is_playwright_test() {
            true
        } else if let Some(key) = lookup.local(path, import) {
            *proofs
                .entry(key.clone())
                .or_insert_with(|| lookup.proves_playwright(&relationships, &key))
        } else {
            false
        };
        let wrappers_ok = wrappers.iter().all(|wrapper| {
            lookup
                .local(path, &wrapper.callee)
                .is_some_and(|key| lookup.proves_wrapper(&key, wrapper.arg_index))
        });
        if base_ok && wrappers_ok {
            verified.insert(item);
        } else if !import.is_playwright_test() {
            if let Some(key) = lookup.local(path, import) {
                let exports: Vec<_> = lookup
                    .reachable_exports(&relationships, &key)
                    .into_iter()
                    .map(|target| (relative_slash_path(root, &target.file), target.name))
                    .collect();
                resolved_map.insert(item, exports);
            }
        }
    }
    for path in spec_files() {
        let Some(file) = facts.get_mut(path) else {
            continue;
        };
        for location in &mut file.test_reliability {
            location.playwright_candidates.retain(|import| {
                let key = (path.clone(), import.clone(), Vec::new());
                if let Some(exports) = resolved_map.get(&key) {
                    location
                        .resolved_playwright_exports
                        .extend(exports.iter().cloned());
                }
                verified.contains(&key)
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
            } else if let Some(exports) = resolved_map.get(&key) {
                location.resolved_playwright_exports = exports.clone();
            }
        }
    }
}

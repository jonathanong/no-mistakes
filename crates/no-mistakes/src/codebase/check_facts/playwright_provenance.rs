use super::{CheckFactPlan, CheckFileFacts, FactRequest};
use crate::codebase::dependencies::extract::is_indexable;
use crate::codebase::testing_analysis::reliability::playwright_provenance::ImportRef;
use crate::codebase::ts_resolver::{
    ScopedImportResolver, TsConfigCatalog, WorkspaceFallbackResolver,
};
use crate::codebase::ts_source::{relative_slash_path, FileIdMap, SourceStore};
use crate::fx::{FxHashMap, FxHashSet, PathSet};
use std::path::{Path, PathBuf};

mod exports;
mod projection;
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
    let workspace = super::workspace_setup::workspace_for(session.session, root, sources);
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
    let mut uncertainties = FxHashMap::default();
    for item in pending {
        let (path, import, wrappers) = &item;
        let key = (!import.is_playwright_test())
            .then(|| lookup.local(path, import))
            .flatten();
        let base_ok = import.is_playwright_test()
            || key.as_ref().is_some_and(|key| {
                *proofs
                    .entry(key.clone())
                    .or_insert_with(|| lookup.proves_playwright(&relationships, key))
            });
        let wrappers_ok = wrappers.iter().all(|wrapper| {
            lookup
                .local(path, &wrapper.callee)
                .is_some_and(|key| lookup.proves_wrapper(&key, wrapper.arg_index))
        });
        if base_ok && wrappers_ok {
            verified.insert(item);
            continue;
        }
        let mut reason_for = |key: &exports::ExportKey| {
            *uncertainties
                .entry(key.clone())
                .or_insert_with(|| lookup.uncertainty_reason(&relationships, key))
        };
        let reason = key
            .as_ref()
            .and_then(&mut reason_for)
            .filter(|reason| *reason != exports::UNRESOLVED_EXPORT)
            .or_else(|| {
                wrappers.iter().find_map(|wrapper| {
                    let key = lookup.local(path, &wrapper.callee)?;
                    reason_for(&key).filter(|reason| *reason != exports::UNRESOLVED_EXPORT)
                })
            })
            .or((!wrappers_ok).then_some("fixture wrapper return could not be traced statically"));
        let exports: Vec<_> = key
            .iter()
            .filter(|_| wrappers_ok)
            .flat_map(|key| lookup.reachable_exports(&relationships, key))
            .map(|target| (relative_slash_path(root, &target.file), target.name))
            .collect();
        resolved_map.insert(item, (reason, exports));
    }
    projection::apply(
        &lookup,
        spec_files(),
        facts,
        &verified,
        &resolved_map,
        &uncertainties,
    );
}

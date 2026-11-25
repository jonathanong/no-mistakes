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
use exports::{ExportLookup, FixtureProof};

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
            file.test_reliability.iter().filter_map(|location| {
                Some((
                    path.clone(),
                    location.playwright_import.as_ref()?.clone(),
                    location.playwright_wrappers.clone(),
                ))
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
    let mut unverified = FxHashMap::default();
    for item in pending {
        let (path, import, wrappers) = &item;
        let mut proof_for = |key: &exports::ExportKey| {
            *proofs
                .entry(key.clone())
                .or_insert_with(|| lookup.fixture_proof(&relationships, key))
        };
        let import_key = (!import.is_playwright_test())
            .then(|| lookup.local(path, import))
            .flatten();
        let proof = if import.is_playwright_test() {
            FixtureProof::Playwright
        } else {
            import_key
                .as_ref()
                .map_or(FixtureProof::Unknown, &mut proof_for)
        };
        let mut reason = proof.unresolved_reason();
        let mut wrappers_ok = true;
        for wrapper in wrappers {
            let Some(key) = lookup.local(path, &wrapper.callee) else {
                wrappers_ok = false;
                continue;
            };
            if !lookup.proves_wrapper(&key, wrapper.arg_index) {
                wrappers_ok = false;
                reason = reason.or_else(|| proof_for(&key).unresolved_reason());
            }
        }
        if matches!(proof, FixtureProof::Playwright) && wrappers_ok {
            verified.insert(item);
            continue;
        }
        let exports: Vec<_> = import_key
            .iter()
            .flat_map(|key| lookup.reachable_exports(&relationships, key))
            .map(|target| (relative_slash_path(root, &target.file), target.name))
            .collect();
        unverified.insert(item, (reason, exports));
    }
    for path in spec_files() {
        let Some(file) = facts.get_mut(path) else {
            continue;
        };
        for location in &mut file.test_reliability {
            let Some(import) = location.playwright_import.clone() else {
                continue;
            };
            let key = (path.clone(), import, location.playwright_wrappers.clone());
            if verified.contains(&key) {
                location.playwright_import = None;
                location.playwright_wrappers.clear();
            } else if let Some((reason, exports)) = unverified.get(&key) {
                location.unresolved_reason = *reason;
                location.resolved_playwright_exports = exports.clone();
            }
        }
    }
}

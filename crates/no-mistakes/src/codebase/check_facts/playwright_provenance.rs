use super::{CheckFactPlan, CheckFileFacts, FactRequest};
use crate::codebase::dependencies::extract::is_indexable;
use crate::codebase::rules::test_reliability::playwright_provenance::ImportRef;
use crate::codebase::ts_resolver::{
    ScopedImportResolver, TsConfigCatalog, WorkspaceFallbackResolver,
};
use crate::codebase::ts_source::{FileIdMap, SourceStore};
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
            file.test_reliability.iter().filter_map(|location| {
                Some((path.clone(), location.playwright_import.as_ref()?.clone()))
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
    let mut wave = pending
        .iter()
        .filter_map(|(path, import)| lookup.local(path, import))
        .collect::<Vec<_>>();
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
    let verified = pending
        .into_iter()
        .filter(|(path, import)| {
            let Some(key) = lookup.local(path, import) else {
                return false;
            };
            *proofs
                .entry(key.clone())
                .or_insert_with(|| lookup.proves_playwright(&relationships, &key))
        })
        .collect::<FxHashSet<_>>();
    for path in spec_files() {
        let Some(file) = facts.get_mut(path) else {
            continue;
        };
        for location in &mut file.test_reliability {
            if location
                .playwright_import
                .as_ref()
                .is_some_and(|import| verified.contains(&(path.clone(), import.clone())))
            {
                location.playwright_import = None;
            }
        }
    }
}

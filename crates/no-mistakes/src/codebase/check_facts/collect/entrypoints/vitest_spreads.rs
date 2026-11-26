use crate::codebase::check_facts::collect_helpers::collect_fact_map_with_sources;
use crate::codebase::check_facts::{CheckFactMap, CheckFactPlan, FactRequest};
use crate::codebase::dependencies::extract::is_indexable;
use crate::codebase::ts_resolver::{ImportResolver, TsConfig};
use crate::codebase::ts_source::SourceStore;
use crate::fx::PathSet;
use std::collections::BTreeSet;
use std::path::Path;
use std::sync::Arc;

pub(super) fn prepare(
    session: FactRequest<'_>,
    root: &Path,
    sources: &SourceStore,
    demand: &CheckFactPlan,
    facts: &mut CheckFactMap,
) {
    let paths = &demand.vitest_default_exclude_paths;
    if paths.is_empty() {
        return;
    }
    let imports: BTreeSet<_> = paths
        .iter()
        .filter_map(|path| Some((path, facts.ts.get(path)?)))
        .filter(|(_, facts)| facts.parse_error.is_none())
        .flat_map(|(path, facts)| {
            let excludes = &facts.vitest_default_excludes;
            excludes
                .default_export
                .iter()
                .chain(&excludes.test_literals)
                .flat_map(move |array| array.imports.iter().map(move |import| (path, import)))
        })
        .collect();
    if imports.is_empty() {
        return;
    }
    let visible: PathSet = sources.inventory().paths().iter().cloned().collect();
    let tsconfig = TsConfig::default();
    let resolver = ImportResolver::new_in_session(&tsconfig, Some(&visible), session.session);
    let imports: Vec<_> = imports
        .into_iter()
        .filter_map(|(from, import)| {
            let target = resolver.resolve(&import.specifier, from)?;
            (target.starts_with(root) && is_indexable(&target)).then_some((from, import, target))
        })
        .collect();
    let helpers: BTreeSet<_> = imports
        .iter()
        .map(|(_, _, target)| target)
        .filter(|target| {
            facts
                .ts
                .get(target)
                .is_none_or(|file| !file.parsed && file.parse_error.is_none())
        })
        .cloned()
        .collect();
    let helpers = collect_fact_map_with_sources(
        session,
        root,
        &helpers.into_iter().collect::<Vec<_>>(),
        demand,
        None,
        sources,
    );
    for (from, import, target) in imports {
        let exports = helpers
            .get(&target)
            .or_else(|| facts.ts.get(&target).map(Arc::as_ref));
        let Some(globs) = exports
            .filter(|facts| facts.parse_error.is_none())
            .and_then(|facts| facts.vitest_default_excludes.exports.get(&import.name))
        else {
            continue;
        };
        facts
            .vitest_exclude_imports
            .entry(from.clone())
            .or_default()
            .insert(import.clone(), globs.clone());
    }
}

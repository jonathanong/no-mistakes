use super::{Compiled, RULE_ID};
use crate::codebase::postgres::statements::{bounded_view_selects, BOUNDED_VIEW_MARKER};
use crate::codebase::postgres::{
    catalog_finding, CatalogIndexInfo, CatalogObjectRef, CatalogView, SchemaCatalog,
};
use crate::codebase::rules::RuleFinding;
use rayon::prelude::*;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

pub(super) fn scan(compiled: Compiled, catalog: &SchemaCatalog) -> Vec<RuleFinding> {
    let problems: BTreeMap<_, _> = catalog
        .views()
        .map(|view| (view.name.as_str(), OnceLock::new()))
        .collect();
    let views: Vec<_> = catalog.views().filter(|view| view.materialized).collect();
    let findings = views.par_iter().flat_map_iter(|view| {
        let indexes: Vec<_> = view.indexes.iter().filter(|index| usable(index)).collect();
        let unique = indexes.iter().any(|index| {
            index.unique
                && index.immediate
                && index.predicate.is_none()
                && !index.keys.is_empty()
                && index.keys.iter().all(|key| key.column.is_some())
        });
        let mut messages = BTreeSet::new();
        if !unique {
            messages.insert("missing a usable, nonpartial unique index on plain columns; add one to support REFRESH MATERIALIZED VIEW CONCURRENTLY".to_string());
        }
        let additional = indexes.len().saturating_sub(usize::from(unique)) as u64;
        if additional < compiled.additional {
            messages.insert(format!("has {additional} additional indexes, needs at least {}; add read indexes for ranking queries", compiled.additional));
        }
        expand(
            &view.name,
            catalog,
            &problems,
            &compiled,
            &mut BTreeSet::new(),
            &mut messages,
        );
        messages.into_iter().map(|message| {
            catalog_finding(
                RULE_ID,
                &compiled.path,
                &CatalogObjectRef::MaterializedView(view.name.clone()),
                &message,
            )
        }).collect::<Vec<_>>()
    }).collect();
    compiled.allow.apply(&compiled.path, findings)
}

pub(super) fn usable(index: &CatalogIndexInfo) -> bool {
    index.valid && index.ready && index.live
}

#[derive(Default)]
struct Problems {
    messages: BTreeSet<String>,
    views: BTreeSet<String>,
}

fn definition_problems(
    compiled: &Compiled,
    catalog: &SchemaCatalog,
    view: &CatalogView,
) -> Problems {
    let mut problems = Problems::default();
    let selects = match bounded_view_selects(view, &compiled.functions, catalog) {
        Ok(selects) => selects,
        Err(error) => {
            problems.messages.insert(format!("cannot analyze definition: {error}; rewrite it as a supported static SELECT so refresh work is bounded, or allow this materialized view with a reason"));
            return problems;
        }
    };
    for select in selects {
        for relation in &select.relations {
            if let Some(view) = catalog.view_relation(&relation.table) {
                problems.views.insert(view.name.clone());
                continue;
            }
            let table = catalog.bounded_relation(&relation.table);
            if compiled.base_tables.iter().any(|entry| {
                table.is_some_and(|table| {
                    catalog
                        .bounded_relation(&entry.table)
                        .is_some_and(|allowed| allowed.name == table.name)
                })
            }) {
                continue;
            }
            let bounded = relation
                .constrained_columns
                .iter()
                .any(|column| column == BOUNDED_VIEW_MARKER);
            if !bounded {
                problems.messages.insert(format!("unbounded base table {}; add a configured window-function predicate on a key or a scalar range on an indexed column, or allowBaseTables with a reason to bound refresh work", relation.table));
            }
        }
    }
    problems
}

fn expand(
    name: &str,
    catalog: &SchemaCatalog,
    problems: &BTreeMap<&str, OnceLock<Problems>>,
    compiled: &Compiled,
    visiting: &mut BTreeSet<String>,
    messages: &mut BTreeSet<String>,
) {
    if !visiting.insert(name.to_string()) {
        messages.insert(format!("cannot prove bounded definition through recursive view {name}; rewrite it statically or allow this materialized view with a reason"));
        return;
    }
    if let Some((view, cell)) = catalog
        .view_relation(name)
        .and_then(|view| problems.get(view.name.as_str()).map(|cell| (view, cell)))
    {
        let problem = cell.get_or_init(|| definition_problems(compiled, catalog, view));
        messages.extend(problem.messages.iter().cloned());
        for child in &problem.views {
            if catalog.view_relation(child).is_some() {
                expand(child, catalog, problems, compiled, visiting, messages);
            }
        }
    }
    visiting.remove(name);
}

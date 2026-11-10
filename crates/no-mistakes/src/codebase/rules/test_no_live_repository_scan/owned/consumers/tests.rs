use super::super::route_index::RouteHit;
use super::Consumed;
use crate::codebase::dependencies::graph::test_support::from_typed_maps;
use crate::codebase::dependencies::graph::TsFactLookup;
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::codebase::ts_source::module_facts::arguments::PathProvenance;
use crate::codebase::ts_source::module_facts::TypeScriptModuleFacts;
use crate::fx::{FxHashMap, FxHashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[test]
fn a_second_targeting_lookup_is_a_cache_hit() {
    let consumed = Consumed::default();
    let file = Path::new("/repo/src/fn.mts");
    assert!(consumed.targeting(file).is_empty());
    assert!(consumed.targeting(file).is_empty());
    let memo = consumed.scan_memo.borrow();
    assert_eq!(memo.walks, 1);
    assert_eq!(memo.hits, 1);
    consumed.report_targeting();
}

#[test]
fn a_private_resolver_follows_a_readable_relative_without_a_session() {
    let importer = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let mut modules = crate::fx::FxHashSet::default();
    modules.insert("./Cargo.toml".to_string());
    let facts = TsFileFacts {
        module_bindings: Some(Arc::new(TypeScriptModuleFacts {
            path_provenance: Some(Arc::new(PathProvenance {
                unsafe_modules: modules,
                ..PathProvenance::default()
            })),
            ..TypeScriptModuleFacts::default()
        })),
        ..TsFileFacts::default()
    };
    let graph = from_typed_maps(
        PathBuf::from("/repo"),
        FxHashMap::default(),
        FxHashMap::default(),
    );
    let resolution = super::Resolution::new(None, None);
    let reachable = FxHashSet::from_iter([importer.clone()]);
    let consumed = Consumed::new(
        &graph,
        &OneFile(&facts),
        std::slice::from_ref(&importer),
        &reachable,
        &resolution,
    );
    assert!(consumed.contains(&importer));
    assert!(!consumed.unknown());
}

struct OneFile<'a>(&'a TsFileFacts);

impl TsFactLookup for OneFile<'_> {
    fn get_ts_facts(&self, _: &Path) -> Option<&TsFileFacts> {
        Some(self.0)
    }
}

impl Consumed {
    pub(in crate::codebase::rules::test_no_live_repository_scan) fn containing(
        file: &Path,
    ) -> Self {
        let mut consumed = Self::default();
        consumed.files.insert(normalize_path(file));
        consumed
    }

    pub(in crate::codebase::rules::test_no_live_repository_scan) fn with_hits(
        hits: Vec<RouteHit>,
    ) -> Self {
        Self {
            hits,
            ..Self::default()
        }
    }
}

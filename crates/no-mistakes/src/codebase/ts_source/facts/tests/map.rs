use super::super::*;
use std::path::{Path, PathBuf};
use std::sync::Arc;

impl TsFactMap {
    pub(crate) fn extend_shared(
        &mut self,
        facts: impl IntoIterator<Item = (PathBuf, Arc<TsFileFacts>)>,
    ) {
        for (path, facts) in facts {
            self.facts.insert(path, TsFactSlot::Shared(facts));
        }
    }

    pub(crate) fn shared_arc(&self, path: &Path) -> Option<&Arc<TsFileFacts>> {
        match self.facts.get(path)? {
            TsFactSlot::Shared(facts) => Some(facts),
            TsFactSlot::Owned(_) => None,
        }
    }

    pub(crate) fn has_owned(&self, path: &Path) -> bool {
        matches!(self.facts.get(path), Some(TsFactSlot::Owned(_)))
    }

    pub(crate) fn shared_is_empty(&self) -> bool {
        self.facts
            .iter()
            .all(|(_, slot)| matches!(slot, TsFactSlot::Owned(_)))
    }
}

#[test]
fn shared_fact_map_reuses_file_fact_allocations() {
    let path = PathBuf::from("/fixture/source.ts");
    let shared = Arc::new(TsFileFacts::default());
    let facts = TsFactMap::from_shared_iter_with_plan(
        [(path.clone(), Arc::clone(&shared))],
        TsFactPlan::default(),
    );

    assert!(Arc::ptr_eq(facts.shared_arc(&path).unwrap(), &shared));
}

#[test]
fn shared_fact_map_materializes_only_mutated_entries() {
    let first_path = PathBuf::from("/fixture/first.ts");
    let second_path = PathBuf::from("/fixture/second.ts");
    let first = Arc::new(TsFileFacts::default());
    let second = Arc::new(TsFileFacts::default());
    let mut facts = TsFactMap::from_shared_iter_with_plan(
        [
            (first_path.clone(), Arc::clone(&first)),
            (second_path.clone(), Arc::clone(&second)),
        ],
        TsFactPlan::default(),
    );

    facts.get_mut(&first_path).unwrap().fatal_parse_error = true;

    assert!(facts.has_owned(&first_path));
    assert!(facts[&first_path].fatal_parse_error);
    assert!(facts.shared_arc(&first_path).is_none());
    assert!(Arc::ptr_eq(
        facts.shared_arc(&second_path).unwrap(),
        &second
    ));
    assert!(!first.fatal_parse_error);
}

#[test]
fn unique_shared_fact_is_unwrapped_on_get_mut() {
    let path = PathBuf::from("/fixture/source.ts");
    let mut facts = TsFactMap::from_shared_iter_with_plan(
        [(
            path.clone(),
            Arc::new(TsFileFacts {
                fatal_parse_error: true,
                ..TsFileFacts::default()
            }),
        )],
        TsFactPlan::default(),
    );

    facts.get_mut(&path).unwrap().fatal_parse_error = false;

    assert!(!facts[&path].fatal_parse_error);
    assert!(facts.has_owned(&path));
}

#[test]
fn unique_shared_facts_are_unwrapped_when_iterating_mutably() {
    let path = PathBuf::from("/fixture/source.ts");
    let mut facts = TsFactMap::from_shared_iter_with_plan(
        [(
            path.clone(),
            Arc::new(TsFileFacts {
                fatal_parse_error: true,
                ..TsFileFacts::default()
            }),
        )],
        TsFactPlan::default(),
    );

    for (_, fact) in &mut facts {
        fact.fatal_parse_error = false;
    }

    assert!(!facts[&path].fatal_parse_error);
    assert!(facts.has_owned(&path));
}

#[test]
fn mixed_fact_map_extension_preserves_new_entry_precedence() {
    let path = PathBuf::from("/fixture/source.ts");
    let shared = Arc::new(TsFileFacts {
        fatal_parse_error: true,
        ..TsFileFacts::default()
    });
    let mut facts =
        TsFactMap::from_shared_iter_with_plan([(path.clone(), shared)], TsFactPlan::default());
    facts.extend(TsFactMap::from([(path.clone(), TsFileFacts::default())]));

    assert!(!facts[&path].fatal_parse_error);
    assert!(facts.shared_is_empty());
    assert_eq!(facts.iter().count(), 1);
}

#[test]
fn shared_fact_map_collection_operations_preserve_map_semantics() {
    let first_path = PathBuf::from("/fixture/first.ts");
    let second_path = PathBuf::from("/fixture/second.ts");
    let mut facts = TsFactMap::from([(first_path.clone(), TsFileFacts::default())]);
    facts.extend(TsFactMap::from_shared_iter_with_plan(
        [(
            first_path.clone(),
            Arc::new(TsFileFacts {
                fatal_parse_error: true,
                ..TsFileFacts::default()
            }),
        )],
        TsFactPlan::default(),
    ));
    facts.extend_shared([(second_path.clone(), Arc::new(TsFileFacts::default()))]);

    assert_eq!(facts.values().count(), 2);
    assert!(facts.remove(&first_path).unwrap().fatal_parse_error);
    for (_, fact) in &mut facts {
        fact.fatal_parse_error = true;
    }
    assert!(facts.remove(&second_path).unwrap().fatal_parse_error);
    assert!(facts.is_empty());
}

#[test]
fn fact_map_get_hits_by_path_after_file_id_indexing() {
    let first = PathBuf::from("/fixture/first.ts");
    let second = PathBuf::from("/fixture/second.ts");
    let mut facts = TsFactMap::from_iter_with_plan(
        [(
            first.clone(),
            TsFileFacts {
                fatal_parse_error: true,
                ..TsFileFacts::default()
            },
        )],
        TsFactPlan::default(),
    );
    facts.insert(second.clone(), TsFileFacts::default());

    assert!(facts.get(&first).unwrap().fatal_parse_error);
    assert!(facts.contains_key(&second));
    assert!(!facts.get(&second).unwrap().fatal_parse_error);
    assert!(facts.get(Path::new("/fixture/missing.ts")).is_none());
}

#[test]
fn borrowed_fact_map_into_iter_stays_lazy() {
    let source = include_str!("../map_iter.rs");
    assert!(
        source.contains("type IntoIter = TsFactMapIter<'a>;"),
        "borrowed TsFactMap iteration must stay a lazy occupied-slot iterator"
    );
    assert!(
        source.contains("type IntoIter = TsFactMapIterMut<'a>;"),
        "mutable TsFactMap iteration must stay a lazy occupied-slot iterator"
    );
}

use super::super::*;
use std::path::{Path, PathBuf};
use std::sync::Arc;

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

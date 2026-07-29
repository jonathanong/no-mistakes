use super::super::*;
use std::path::PathBuf;

impl TsFactMap {
    pub(crate) fn extend_shared(
        &mut self,
        facts: impl IntoIterator<Item = (PathBuf, std::sync::Arc<TsFileFacts>)>,
    ) {
        self.shared.extend(facts);
    }
}

#[test]
fn shared_fact_map_reuses_file_fact_allocations() {
    let path = PathBuf::from("/fixture/source.ts");
    let shared = std::sync::Arc::new(TsFileFacts::default());
    let facts = TsFactMap::from_shared_iter_with_plan(
        [(path.clone(), std::sync::Arc::clone(&shared))],
        TsFactPlan::default(),
    );

    assert!(std::sync::Arc::ptr_eq(
        facts.shared.get(&path).unwrap(),
        &shared
    ));
}

#[test]
fn shared_fact_map_materializes_only_mutated_entries() {
    let first_path = PathBuf::from("/fixture/first.ts");
    let second_path = PathBuf::from("/fixture/second.ts");
    let first = std::sync::Arc::new(TsFileFacts::default());
    let second = std::sync::Arc::new(TsFileFacts::default());
    let mut facts = TsFactMap::from_shared_iter_with_plan(
        [
            (first_path.clone(), std::sync::Arc::clone(&first)),
            (second_path.clone(), std::sync::Arc::clone(&second)),
        ],
        TsFactPlan::default(),
    );

    facts.get_mut(&first_path).unwrap().fatal_parse_error = true;

    assert!(facts.owned[&first_path].fatal_parse_error);
    assert!(!facts.shared.contains_key(&first_path));
    assert!(std::sync::Arc::ptr_eq(
        facts.shared.get(&second_path).unwrap(),
        &second
    ));
}

#[test]
fn mixed_fact_map_extension_preserves_new_entry_precedence() {
    let path = PathBuf::from("/fixture/source.ts");
    let shared = std::sync::Arc::new(TsFileFacts {
        fatal_parse_error: true,
        ..TsFileFacts::default()
    });
    let mut facts =
        TsFactMap::from_shared_iter_with_plan([(path.clone(), shared)], TsFactPlan::default());
    facts.extend(TsFactMap::from([(path.clone(), TsFileFacts::default())]));

    assert!(!facts[&path].fatal_parse_error);
    assert!(facts.shared.is_empty());
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
            std::sync::Arc::new(TsFileFacts {
                fatal_parse_error: true,
                ..TsFileFacts::default()
            }),
        )],
        TsFactPlan::default(),
    ));
    facts.extend_shared([(
        second_path.clone(),
        std::sync::Arc::new(TsFileFacts::default()),
    )]);

    assert_eq!(facts.values().count(), 2);
    assert!(facts.remove(&first_path).unwrap().fatal_parse_error);
    for (_, fact) in &mut facts {
        fact.fatal_parse_error = true;
    }
    assert!(facts.remove(&second_path).unwrap().fatal_parse_error);
    assert!(facts.is_empty());
}

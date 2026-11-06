use super::super::super::{
    budget::{Cost, Limits},
    lower::Lowered,
};
use super::super::{Catalog, Engine, Value};
use super::{Cache, Key, Summary};
use crate::codebase::dependencies::extract::CallableId;
use rustc_hash::FxHasher;
use std::{
    hash::{Hash, Hasher},
    path::Path,
};

struct EmptyCatalog;
impl Catalog for EmptyCatalog {
    fn function(&self, _: &Key) -> Option<&Lowered> {
        None
    }
    fn callee(&self, _: &Path, _: usize) -> Option<Key> {
        None
    }
    fn sink(&self, _: &Path, _: usize) -> bool {
        false
    }
    fn factory(&self, _: &Path, _: usize) -> bool {
        false
    }
    fn value_throw(&self, _: &Path, _: (u32, u32)) -> bool {
        false
    }
    fn local(&self, _: &Path, _: usize, _: (u32, u32)) -> Option<&Lowered> {
        None
    }
}

#[test]
fn cached_unknown_replays_the_consulted_source_failure() {
    let failed_source = crate::test_support::rule_fixture_root("postgres-no-offset")
        .join("low-helper-recovery/src/failure-helper.mts");
    let key = Key {
        file: failed_source.clone(),
        callable: CallableId(7),
    };
    let mut hasher = FxHasher::default();
    false.hash(&mut hasher);
    Option::<(u32, u32)>::None.hash(&mut hasher);
    let mut cache = Cache::default();
    cache.complete.insert(
        (key.clone(), hasher.finish()),
        vec![Summary {
            args: Vec::new(),
            outcomes: vec![Value::Scalar],
            cost: Cost::default(),
            executed: Vec::new(),
            failed_sources: [failed_source.clone()].into_iter().collect(),
        }],
    );

    let result = Engine::new(&EmptyCatalog, &mut cache, Limits::default())
        .run(&key, &[])
        .unwrap();

    assert!(matches!(result.as_slice(), [Value::Scalar]));
    assert_eq!(
        cache
            .take_observed_sources()
            .into_iter()
            .collect::<Vec<_>>(),
        [failed_source]
    );
}

use super::*;
use crate::fx::FxHashSet;
use std::{cell::RefCell, collections::HashMap, ops::DerefMut, sync::Arc};

type BuilderIndexes = HashMap<usize, (FxHashMap<String, Value>, Arc<FxHashSet<u64>>)>;

thread_local! {
    static BUILDER_INDEXES: RefCell<BuilderIndexes> = RefCell::new(HashMap::new());
}

impl Scope {
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn shares_storage_with(
        &self,
        other: &Self,
    ) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }

    pub(in crate::codebase::postgres::query_annotation::evaluate) fn builder_index_initialized(
        &self,
    ) -> bool {
        let key = Arc::as_ptr(&self.0) as usize;
        BUILDER_INDEXES.with(|indexes| {
            indexes
                .borrow()
                .get(&key)
                .is_some_and(|(values, _)| values == &self.0.values)
        })
    }

    pub(in crate::codebase::postgres::query_annotation::evaluate) fn contains_builders(
        &self,
        ids: &FxHashSet<u64>,
    ) -> bool {
        let key = Arc::as_ptr(&self.0) as usize;
        let values = self.0.values.clone();
        let builders = BUILDER_INDEXES.with(|indexes| {
            let mut indexes = indexes.borrow_mut();
            if indexes
                .get(&key)
                .is_none_or(|(cached, _)| cached != &values)
            {
                let mut found = FxHashSet::default();
                let mut visited = FxHashSet::default();
                for value in values.values() {
                    collect_builders(value, &mut found, &mut visited);
                }
                indexes.insert(key, (values, Arc::new(found)));
            }
            indexes.get(&key).expect("builder index inserted").1.clone()
        });
        !builders.is_disjoint(ids)
    }
}

fn collect_builders(value: &Value, found: &mut FxHashSet<u64>, visited: &mut FxHashSet<*const ()>) {
    match value {
        Value::Prefix(_, _, Some(id)) => {
            found.insert(*id);
        }
        Value::Promise(value) | Value::Evaluated(value, _) => {
            collect_builders(value, found, visited)
        }
        Value::Aggregate(values)
        | Value::References(values)
        | Value::Object(values)
        | Value::Record(values, _)
        | Value::Joined(values)
        | Value::Possible(values)
            if values.contains_reference() && visited.insert(values.identity()) =>
        {
            for value in values {
                collect_builders(value, found, visited);
            }
        }
        _ => {}
    }
}

fn builder_index_identity(scope: &Scope) -> usize {
    let key = Arc::as_ptr(&scope.0) as usize;
    BUILDER_INDEXES.with(|indexes| {
        Arc::as_ptr(
            &indexes
                .borrow()
                .get(&key)
                .expect("builder index initialized")
                .1,
        ) as usize
    })
}

impl<'a> IntoIterator for &'a mut Scope {
    type Item = (&'a String, &'a mut Value);
    type IntoIter = std::collections::hash_map::IterMut<'a, String, Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl DerefMut for Scope {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut Arc::make_mut(&mut self.0).values
    }
}

#[test]
fn builder_index_reuses_snapshots_and_follows_every_write() {
    let prefix = |id| Value::Prefix("SELECT 1".into(), true, Some(id));
    let ids = FxHashSet::from_iter([7]);
    let mut scope = Scope::from(FxHashMap::from_iter([
        ("direct".into(), prefix(7)),
        (
            "nested".into(),
            Value::Promise(Box::new(Value::Evaluated(
                Box::new(Value::Aggregate(
                    vec![Value::Possible(
                        vec![Value::Joined(vec![prefix(8)].into())].into(),
                    )]
                    .into(),
                )),
                true,
            ))),
        ),
        ("plain".into(), Value::Unknown),
    ]));
    let snapshot = scope.clone();
    assert!(scope.contains_builders(&ids));
    let index = builder_index_identity(&scope);
    assert!(snapshot.contains_builders(&FxHashSet::from_iter([8])));
    assert_eq!(index, builder_index_identity(&snapshot));
    assert!(!scope.contains_builders(&FxHashSet::from_iter([9])));
    assert!(scope == snapshot);
    assert_eq!((&scope).into_iter().count(), 3);

    scope.insert("direct".into(), Value::Unknown);
    assert!(!scope.contains_builders(&ids));
    assert!(snapshot.contains_builders(&ids));
    assert!(scope != snapshot);
    for (_, value) in &mut scope {
        *value = Value::Unknown;
    }
    assert!(!scope.contains_builders(&FxHashSet::from_iter([8])));
    // Unique-owner writes must clear the same index as detached writes.
    scope.insert("fresh".into(), prefix(9));
    assert!(scope.contains_builders(&FxHashSet::from_iter([9])));
    assert!(!Scope::default().contains_builders(&ids));
}

#[test]
fn builder_index_handles_shared_container_dag_and_detached_mutation() {
    let leaf = Value::Possible(vec![Value::Prefix("SELECT 1".into(), true, Some(17))].into());
    let branch = Value::Aggregate(vec![leaf; 16].into());
    let mut scope = Scope::from(FxHashMap::from_iter([(
        "shared".into(),
        Value::Joined(vec![branch; 16].into()),
    )]));
    let original = scope.clone();
    let old = FxHashSet::from_iter([17]);
    assert!(scope.contains_builders(&old));
    assert!(original.contains_builders(&old));

    scope.insert(
        "new".into(),
        Value::Prefix("SELECT 2".into(), true, Some(18)),
    );
    let new = FxHashSet::from_iter([18]);
    assert!(scope.contains_builders(&new));
    assert!(!original.contains_builders(&new));
    assert!(original.contains_builders(&old));
}

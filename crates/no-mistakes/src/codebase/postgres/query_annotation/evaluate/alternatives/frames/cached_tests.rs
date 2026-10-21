use super::*;
use crate::codebase::postgres::query_annotation::evaluate::ArgumentObjects;
use std::collections::BTreeMap;

#[derive(Clone, Default)]
struct State {
    scopes: Vec<Scope>,
    objects: ArgumentObjects,
    extras: FxHashMap<u64, BTreeMap<usize, Value>>,
    mapped: FxHashMap<Environment, Vec<(u64, Vec<String>)>>,
    fresh: FxHashMap<Environment, FxHashSet<String>>,
    captured: FxHashMap<Environment, FxHashMap<String, Environment>>,
    returned: Vec<Value>,
    modules:
        FxHashMap<crate::codebase::postgres::query_annotation::module_ids::ModuleId, Environment>,
}

impl State {
    fn run(&mut self, cached: bool, prefix: &mut reachable::Prefix) -> Compaction {
        let arena = ArenaMut {
            objects: &mut self.objects,
            extras: &mut self.extras,
        };
        if cached {
            compact(
                &mut self.scopes,
                ModuleRoots {
                    original: 1,
                    prefix,
                    modules: &mut self.modules,
                    initials: &mut [],
                    transient: &[],
                },
                &mut self.returned,
                &mut self.mapped,
                arena,
                &mut self.fresh,
                &mut self.captured,
            )
        } else {
            let mut roots = vec![0];
            roots.extend(self.modules.values().copied());
            let result = compact_from_roots(
                &mut self.scopes,
                Roots {
                    environments: &roots,
                    arguments: &[],
                    preserved: 1,
                    transient: &[],
                },
                &mut self.returned,
                &mut self.mapped,
                arena,
                &mut self.fresh,
                &mut self.captured,
            );
            for env in self.modules.values_mut() {
                if let Some(new) = result.indices.get(env) {
                    *env = *new;
                }
            }
            result
        }
    }

    fn same(&self, other: &Self) {
        assert!(self.scopes == other.scopes);
        assert!(self.objects == other.objects);
        assert!(self.extras == other.extras);
        assert_eq!(self.mapped, other.mapped);
        assert_eq!(self.fresh, other.fresh);
        assert_eq!(self.captured, other.captured);
        assert!(self.returned == other.returned);
        assert_eq!(self.modules, other.modules);
    }
}

#[test]
fn exact_compacted_state_matches_legacy_through_remaps_and_root_removal() {
    let mut state = State {
        scopes: vec![
            FxHashMap::from_iter([("args".into(), Value::Arguments(7))]).into(),
            Scope::default(),
            Scope::default(),
            FxHashMap::from_iter([(
                "sql".into(),
                Value::Prefix("SELECT 1".into(), true, Some(19)),
            )])
            .into(),
        ],
        objects: ArgumentObjects::from_iter([
            (7, vec![Value::Arguments(7)]),
            (8, vec![Value::Arguments(8)]),
            (99, vec![]),
        ]),
        extras: FxHashMap::from_iter([(7, BTreeMap::from([(5, Value::Array(8))]))]),
        mapped: FxHashMap::from_iter([(3, vec![(8, vec!["mapped".into()])])]),
        fresh: FxHashMap::from_iter([(3, FxHashSet::from_iter(["mapped".into()]))]),
        captured: FxHashMap::from_iter([(0, FxHashMap::from_iter([("source".into(), 3)]))]),
        modules: FxHashMap::from_iter([(
            crate::codebase::postgres::query_annotation::module_ids::ModuleIds::new([
                "module.mts".into()
            ])
            .id(std::path::Path::new("module.mts")),
            3,
        )]),
        ..Default::default()
    };
    let mut old = state.clone();
    let mut prefix = reachable::Prefix::default();
    for round in 0..3 {
        if round == 2 {
            state.scopes[0].clear();
            state.captured.clear();
            state.modules.clear();
            old = state.clone();
        }
        let expected = old.run(false, &mut reachable::Prefix::default());
        let actual = state.run(true, &mut prefix);
        assert_eq!(expected.indices, actual.indices);
        assert_eq!(expected.arguments, actual.arguments);
        assert_eq!(expected.identities, actual.identities);
        assert_eq!(expected.captured_changed, actual.captured_changed);
        state.same(&old);
    }
    assert_eq!(state.scopes.len(), 1);
    assert!(state.objects.is_empty());
    assert!(state.mapped.is_empty());
    assert!(state.fresh.is_empty());
}

use super::*;
use crate::codebase::postgres::query_annotation::evaluate::{
    temporaries::TransientRoot, ArgumentObjects,
};
use std::collections::BTreeMap;
#[path = "tests/summary_fixture.rs"]
mod fixture;
#[path = "tests/legacy.rs"]
mod legacy;

#[derive(Clone, Default)]
struct State {
    scopes: Vec<Scope>,
    objects: ArgumentObjects,
    extras: FxHashMap<u64, BTreeMap<usize, Value>>,
    mapped: FxHashMap<Environment, Vec<(u64, Vec<String>)>>,
    captured: CapturedBindings,
    returned: Vec<Value>,
    transient: Vec<TransientRoot>,
    deleted: FxHashSet<(u64, Option<usize>)>,
    definite: FxHashSet<(u64, usize)>,
    disconnected: FxHashSet<(u64, usize)>,
    invalidated: FxHashSet<u64>,
    updates: FxHashMap<u64, Value>,
}
impl State {
    fn prune(&mut self, raw: bool) -> usize {
        if raw {
            let roots = (0..self.scopes.len()).collect::<Vec<_>>();
            let live = legacy::collect_from_roots(
                &self.scopes,
                legacy::RootSeeds {
                    environments: &roots,
                    transient: &self.transient,
                    ..Default::default()
                },
                &self.returned,
                super::super::super::arena::Arena {
                    objects: &self.objects,
                    extras: &self.extras,
                },
                &self.mapped,
                &self.captured,
            );
            self.objects.retain(|id, _| live.arguments.contains(id));
            self.extras.retain(|id, _| live.arguments.contains(id));
            self.deleted.retain(|(id, _)| live.arguments.contains(id));
            self.definite.retain(|(id, _)| live.arguments.contains(id));
            self.disconnected
                .retain(|(id, _)| live.arguments.contains(id));
            self.invalidated.retain(|id| live.identities.contains(id));
            self.updates.retain(|id, _| live.identities.contains(id));
            live.visits
        } else {
            prune_state(
                &self.scopes,
                &self.returned,
                MutationState {
                    deleted: &mut self.deleted,
                    definite: &mut self.definite,
                    disconnected: &mut self.disconnected,
                    invalidated: &mut self.invalidated,
                    updates: &mut self.updates,
                },
                &self.mapped,
                ArenaMut {
                    objects: &mut self.objects,
                    extras: &mut self.extras,
                },
                &self.captured,
                &self.transient,
            );
            0
        }
    }
    fn same(&self, other: &Self) {
        macro_rules! equal { ($($field:ident),+) => { $(assert!(self.$field == other.$field, "{}", stringify!($field));)+ }; }
        let roots = |values: &[TransientRoot]| {
            values
                .iter()
                .map(|root| match root {
                    TransientRoot::Argument(id) => (0, *id),
                    TransientRoot::Builder(id) => (1, *id),
                    TransientRoot::Environment(env) => (2, *env as u64),
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(roots(&self.transient), roots(&other.transient));
        equal!(
            scopes,
            objects,
            extras,
            mapped,
            captured,
            returned,
            deleted,
            definite,
            disconnected,
            invalidated,
            updates
        );
    }
}
#[derive(Debug)]
struct Work {
    legacy_visits: usize,
    bindings: usize,
    slots: usize,
    values: usize,
    summary_scans: u64,
}
fn compare(state: &State) -> Work {
    let (mut raw, mut summarized) = (state.clone(), state.clone());
    let legacy_visits = raw.prune(true);
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let _observer = crate::diagnostics::InvocationGuard::install(observer.clone());
    let scans = reachable::scan_hooks::Guard::new();
    summarized.prune(false);
    raw.same(&summarized);
    let (bindings, slots, values) = scans.counts();
    let summary_scans = observer
        .snapshot()
        .work
        .iter()
        .filter(|(name, _)| {
            matches!(
                **name,
                "postgres.annotation.prefix_binding_scans"
                    | "postgres.annotation.prefix_scope_scans"
                    | "postgres.annotation.dense_reference_slots_scanned"
            )
        })
        .map(|(_, count)| *count)
        .sum();
    assert_eq!(
        summary_scans, 0,
        "pruning must never initialize a cold summary"
    );
    Work {
        legacy_visits,
        bindings,
        slots,
        values,
        summary_scans,
    }
}
#[path = "tests/warm_cases.rs"]
mod warm_cases;

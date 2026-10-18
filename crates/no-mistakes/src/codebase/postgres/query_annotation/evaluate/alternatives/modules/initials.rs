//! Lazy module initialization promotes a pristine generation into active arm baselines.
use super::super::super::{frame_arena::FrameCheckpoint, Environment, Evaluator};
use super::super::{arena::Arena, reachable};
use crate::fx::FxHashSet;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

#[derive(Default)]
pub(in crate::codebase::postgres::query_annotation) struct Initials {
    pub base: FrameCheckpoint,
    known: FxHashSet<PathBuf>,
    frames: BTreeMap<u64, Environment>,
    objects: BTreeSet<u64>,
    identities: FxHashSet<u64>,
    modules: FxHashSet<Environment>,
}

impl Initials {
    pub fn young_arguments(&self, cutoff: u64) -> impl Iterator<Item = u64> + '_ {
        self.objects.range(cutoff..).copied()
    }
    pub fn young_roots(&self, base: FrameCheckpoint) -> impl Iterator<Item = Environment> + '_ {
        self.frames.range(base.birth..).map(|(_, env)| *env)
    }
}

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(in crate::codebase::postgres::query_annotation::evaluate::alternatives) fn begin_alternative_module_snapshot(
        &mut self,
        base: FrameCheckpoint,
    ) {
        self.active_module_initials.push(Initials {
            base,
            known: Default::default(),
            frames: Default::default(),
            objects: Default::default(),
            identities: Default::default(),
            modules: Default::default(),
        });
    }

    pub(in crate::codebase::postgres::query_annotation::evaluate) fn record_alternative_module_initial(
        &mut self,
        path: &Path,
        env: Environment,
    ) {
        if self.active_module_initials.is_empty() {
            return;
        }
        if self.active_module_initials.iter().all(|snapshot| {
            self.scopes.is_before(env, snapshot.base) || snapshot.known.contains(path)
        }) {
            return;
        }
        let live = reachable::collect_from_roots(
            &self.scopes,
            reachable::RootSeeds {
                environments: &[env],
                ..Default::default()
            },
            &[],
            Arena {
                objects: &self.argument_objects,
                extras: &self.argument_extra_slots,
            },
            &self.mapped_arguments,
            &self.captured_bindings,
        );
        let mut promoted_frames = FxHashSet::default();
        let mut pins = Vec::new();
        for snapshot in &mut self.active_module_initials {
            if self.scopes.is_before(env, snapshot.base)
                || !snapshot.known.insert(path.to_path_buf())
            {
                continue;
            }
            snapshot.modules.insert(env);
            let frames = live
                .environments
                .iter()
                .filter(|frame| !self.scopes.is_before(**frame, snapshot.base));
            for frame in frames {
                snapshot.frames.insert(self.scopes.birth(*frame), *frame);
                promoted_frames.insert(*frame);
            }
            snapshot.objects.extend(live.arguments.iter().copied());
            for id in &live.identities {
                if snapshot.identities.insert(*id) {
                    pins.push(*id);
                }
            }
        }
        self.pin_module_identities(&pins);
        self.promote_module_initial(
            path,
            &promoted_frames.into_iter().collect::<Vec<_>>(),
            &live.arguments.into_iter().collect::<Vec<_>>(),
            &live.identities.into_iter().collect::<Vec<_>>(),
        );
        if let Some(observer) = crate::diagnostics::current() {
            let oldest = self
                .active_module_initials
                .first()
                .expect("active snapshots")
                .base;
            let old_visits = live
                .environments
                .iter()
                .filter(|env| self.scopes.is_before(**env, oldest))
                .count();
            observer.increment("postgres.annotation.module_initial_promotions", 1);
            observer.increment(
                "postgres.annotation.module_initial_old_frame_visits",
                old_visits as u64,
            );
            observer.increment(
                "postgres.annotation.module_initial_new_frame_visits",
                (live.environments.len() - old_visits) as u64,
            );
        }
    }
    pub(in crate::codebase::postgres::query_annotation::evaluate::alternatives) fn finish_alternative_module_snapshot(
        &mut self,
    ) -> Vec<super::super::super::temporaries::TransientRoot> {
        let snapshot = self
            .active_module_initials
            .pop()
            .expect("balanced alternatives");
        self.unpin_module_identities(&snapshot.identities.into_iter().collect::<Vec<_>>());
        snapshot
            .modules
            .into_iter()
            .map(super::super::super::temporaries::TransientRoot::Environment)
            .collect()
    }
}

#[cfg(test)]
#[path = "initials/test_accessors.rs"]
mod test_accessors;
#[cfg(test)]
mod tests;

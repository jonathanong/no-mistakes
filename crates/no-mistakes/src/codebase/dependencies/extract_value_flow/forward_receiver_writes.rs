use super::*;
use mutation_projection::{MutationInfo, PendingQueue};
#[path = "forward_parameter_arguments.rs"]
mod parameter_arguments;

/// Only unresolved lexical receivers retain work. Group shared alias paths by
/// physical writer, then reuse the ordinary visited projection after collection.
#[derive(Default)]
pub(in super::super) struct ForwardWrites {
    completed: bool,
    writers: Vec<ForwardWrite>,
    indexes: FxHashMap<(u32, Option<CallableId>, u64), usize>,
}
struct ForwardWrite {
    sources: Vec<mutation_projection::PendingMutation<nested::RootRead>>,
    metadata: mutation_metadata::MutationMetadata,
    member: Option<String>,
    completed_at: u32,
    assigned: bool,
    proof: mutable_deferred::DeferredProof,
}

impl ImportCollector {
    pub(super) fn defer_forward_receiver_write(
        &mut self,
        key: &(usize, String),
        path: &[Option<String>],
        reads: &[nested::ReceiverRead],
        read: Option<nested::RootRead>,
        info: MutationInfo<'_>,
        propagated: bool,
    ) -> bool {
        if info.metadata.context.is_none()
            || self
                .forward_receiver_writes
                .as_ref()
                .is_some_and(|work| work.completed)
            || (!self.has_lexical_binding_at(key.0, &key.1)
                && !self
                    .value_parameter_indexes
                    .get(&key.0)
                    .is_some_and(|names| names.contains_key(key.1.as_str())))
            || self.has_mutable_receiver_binding(key.0, &key.1)
            || self
                .value_binding_indexes
                .get(&key.0)
                .is_some_and(|names| names.contains_key(key.1.as_str()))
        {
            return false;
        }
        let work = self
            .forward_receiver_writes
            .get_or_insert_with(Default::default);
        let physical = (
            info.metadata.offset,
            info.metadata.context,
            info.metadata.epoch,
        );
        let index = *work.indexes.entry(physical).or_insert_with(|| {
            let index = work.writers.len();
            work.writers.push(ForwardWrite {
                sources: Vec::new(),
                metadata: info.metadata,
                member: info.assigned_member.map(str::to_owned),
                completed_at: info.completed_at,
                assigned: info.has_assignment,
                proof: mutable_deferred::DeferredProof {
                    phases: self.fresh_receiver_effects.phases.clone(),
                    region: self
                        .receiver_conditional_region
                        .map(|(region, _)| region.get()),
                    owner: self.destructuring_phase_owner,
                },
            });
            index
        });
        work.writers[index].sources.push((
            (
                (key.clone(), path.to_vec()),
                reads.to_vec(),
                read.unwrap_or((info.metadata.offset, info.metadata.context)),
            ),
            false,
            false,
            propagated,
        ));
        #[cfg(feature = "test-instrumentation")]
        {
            self.value_flow.reference_forward_sources += 1;
        }
        true
    }

    pub(in super::super) fn finish_forward_receiver_writes(&mut self) {
        // Final aliases own argument recipients before either writer channel replays.
        self.finish_possible_arguments();
        let Some(mut work) = self.forward_receiver_writes.take() else {
            return;
        };
        work.completed = true;
        let writers = std::mem::take(&mut work.writers);
        work.indexes.clear();
        self.forward_receiver_writes = Some(work);
        let mut alias_index = None;
        let mut parameter_arguments = None;
        for mut writer in writers {
            writer
                .sources
                .retain(|((((scope, name), _), _, _), _, _, _)| {
                    self.value_binding_indexes
                        .get(scope)
                        .is_some_and(|names| names.contains_key(name.as_str()))
                        || self.has_mutable_receiver_binding(*scope, name)
                        || self
                            .value_parameter_indexes
                            .get(scope)
                            .is_some_and(|names| names.contains_key(name.as_str()))
                });
            if writer.sources.is_empty() {
                continue;
            }
            let mut possible_sources = Vec::new();
            if writer
                .sources
                .iter()
                .any(|((((scope, name), _), _, _), _, _, _)| {
                    self.value_parameter_indexes
                        .get(scope)
                        .is_some_and(|names| names.contains_key(name.as_str()))
                })
            {
                let arguments =
                    parameter_arguments.get_or_insert_with(|| self.parameter_call_origins());
                let mut definite_sources = Vec::new();
                for source in writer.sources {
                    let (definite, possible) = parameter_arguments::expand(arguments, source);
                    definite_sources.extend(definite);
                    possible_sources.extend(possible);
                }
                writer.sources = definite_sources;
            }
            self.replay_forward_write(&mut writer, possible_sources, &mut alias_index);
        }
    }
}

#[cfg(all(test, feature = "test-instrumentation"))]
#[path = "forward_receiver_work_tests.rs"]
mod tests;

#[path = "forward_receiver_replay.rs"]
mod replay;

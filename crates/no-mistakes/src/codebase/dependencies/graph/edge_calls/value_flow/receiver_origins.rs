use super::*;
use default_origins::{merge, ReceiverRead};

impl ValueFlowResolution<'_, '_> {
    pub(super) fn record_receiver_origin(
        &mut self,
        file: &std::path::Path,
        scope: usize,
        name: &str,
        phase: (u32, Option<CallableId>, bool),
    ) {
        let (offset, context, repeated) = phase;
        if self.indexes.file(self.facts, file).is_some_and(|index| {
            index
                .values
                .receiver_mutations
                .as_ref()
                .and_then(|receivers| receivers.get(&scope))
                .and_then(|names| names.get(name))
                .is_some_and(|history| history.has_assignments)
        }) {
            let source = self.cache.origin_node(
                OriginKind::Receiver(file.to_path_buf(), scope, name.to_string()),
                1,
            );
            let source = self.cache.origin_node(
                OriginKind::ReceiverUse(
                    Some(source),
                    ReceiverRead {
                        file: file.to_path_buf(),
                        offset,
                        context,
                        invocation: false,
                        repeated,
                        owned_context: None,
                    },
                ),
                1,
            );
            let source = if let Some(frame) = self.owned_receiver_context(file, context) {
                self.cache
                    .origin_node(OriginKind::ReceiverContexts(source, frame), 1)
            } else {
                source
            };
            merge(&mut self.default_origins, Some(source), self.cache);
        }
    }
    fn owned_receiver_context(
        &self,
        file: &std::path::Path,
        context: Option<CallableId>,
    ) -> default_origins::Origins {
        let owner = context?;
        let frame = self.acquisition_context.as_ref()?;
        Some(self.cache.origin_node(
            OriginKind::ReceiverOwnedContexts(
                std::sync::Arc::clone(frame),
                file.to_path_buf(),
                owner,
            ),
            1,
        ))
    }
    pub(super) fn receiver_field_value(
        &mut self,
        file: &std::path::Path,
        expression: &ValueExpression,
    ) -> FlowValue {
        let context = self
            .default_origins
            .as_ref()
            .filter(|origins| origins.has_receiver_reads)
            .cloned();
        // Finish acquisition first: primary.get copied into an earlier object
        // is already a method snapshot; primary remains a live receiver.
        let previous = self.acquisition_context.take();
        self.acquisition_context = context.clone().or_else(|| previous.clone());
        let value = self.value(file, expression);
        if let Some((read_file, offset, read_context)) = self.member_read.clone() {
            self.record_value_receiver_read(&read_file, offset, read_context);
        }
        if self.acquisition_context.is_some() {
            self.seal_receiver_acquisition();
        }
        self.acquisition_context = previous;
        if let Some(context) = context {
            if let Some(origins) = self.default_origins.take() {
                self.default_origins = Some(
                    self.cache
                        .origin_node(OriginKind::ReceiverContexts(origins, context), 1),
                );
            }
        }
        value
    }
    pub(super) fn seal_receiver_acquisition(&mut self) {
        if let Some(origins) = self.default_origins.take() {
            self.default_origins = Some(if origins.has_unsealed_owned_contexts {
                self.cache
                    .origin_node(OriginKind::ReceiverAcquisition(origins), 1)
            } else {
                origins
            });
        }
    }
    pub(super) fn snapshot_receiver_origins(&mut self) {
        if let Some(origins) = self.default_origins.take() {
            if !origins.has_receivers {
                self.default_origins = Some(origins);
                return;
            }
            self.default_origins = Some(
                self.cache
                    .origin_node(OriginKind::ReceiverSnapshot(origins), 1),
            );
        }
    }
    pub(super) fn record_receiver_use(
        &mut self,
        file: &std::path::Path,
        call: &ValueCall,
        value: &FlowValue,
        owned_context: Option<(&std::path::Path, CallableId, bool)>,
    ) {
        let lazy = match value.executor() {
            FlowValue::Object { file, .. } | FlowValue::Array { file, .. } => self
                .indexes
                .file(self.facts, file)
                .is_some_and(|index| index.values.receiver_mutations.is_some()),
            _ => false,
        };
        if !lazy
            && !self
                .default_origins
                .as_ref()
                .is_some_and(|origins| origins.has_receivers || origins.has_receiver_reads)
        {
            return;
        }
        let origins = self.default_origins.take();
        self.default_origins = Some(self.make_receiver_use(file, call, origins, owned_context));
    }
    pub(super) fn make_receiver_use(
        &self,
        file: &std::path::Path,
        call: &ValueCall,
        origins: default_origins::Origins,
        owned_context: Option<(&std::path::Path, CallableId, bool)>,
    ) -> std::sync::Arc<OriginNode> {
        let (offset, context, repeated) =
            self.expression_read(file, call.end.saturating_sub(1), call.context);
        self.cache.origin_node(
            OriginKind::ReceiverUse(
                origins,
                ReceiverRead {
                    file: file.to_path_buf(),
                    // The closing delimiter follows argument evaluation, but precedes
                    // an enclosing assignment whose exclusive end equals call.end.
                    offset,
                    // Inline callees have no Named expression; facts own their caller context.
                    context,
                    invocation: true,
                    // Named-only invocation timing retains its existing policy.
                    // Inline entry and qualified caller proofs own this phase.
                    repeated: repeated
                        || (call.repeated && super::inline_inputs::inline_callee(&call.callee)),
                    owned_context: owned_context.map(|(file, owner, preserve)| {
                        Box::new((file.to_path_buf(), owner, preserve))
                    }),
                },
            ),
            1,
        )
    }
}

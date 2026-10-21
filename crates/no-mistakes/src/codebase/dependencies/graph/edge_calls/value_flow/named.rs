use super::*;

impl ValueFlowResolution<'_, '_> {
    pub(super) fn named(
        &mut self,
        path: &std::path::Path,
        name: &str,
        scope: Option<usize>,
        offset: u32,
        context: Option<CallableId>,
        projection: Option<&member_projection::MemberProjection<'_>>,
    ) -> FlowValue {
        if let Some(value) = self.replacement_value(path, name, scope, offset, context) {
            return value;
        }
        let source_offset = offset;
        let source_context = context;
        // Runtime body reads and supplied acquisition are distinct; the
        // invocation identity still owns the separate escape guard below.
        let (offset, context, repeated) = self.expression_read(path, offset, context);
        let Some(file) = self.indexes.file(self.facts, path) else {
            return FlowValue::Unknown;
        };
        let (base, member) = name
            .split_once('.')
            .map_or((name, None), |(base, member)| (base, Some(member)));
        let Some(scope) = scope.or_else(|| file.imported.contains_key(base).then_some(0)) else {
            return FlowValue::Unknown;
        };
        let key = (path.to_path_buf(), scope, base.to_string());
        let parameter = scoped_name(&file.values.parameters, scope, base);
        if let Some(parameter) = parameter {
            self.used_parameters
                .insert((path.to_path_buf(), parameter.owner));
        }
        if !self.visiting.insert(key.clone()) {
            return FlowValue::Unknown;
        }
        let binding = scoped_name(&file.values.bindings, scope, base);
        let value = if let Some(binding) = binding {
            self.binding_value(
                path,
                binding,
                (source_offset, source_context),
                (offset, context),
                projection,
            )
        } else if let Some(parameter) = parameter {
            if self.initializing.iter().any(|(file, owner, index)| {
                file == path
                    && *owner == parameter.owner
                    && parameter.initialization_order >= *index
            }) || parameter
                .mutation_contexts
                .iter()
                .any(|writer| *writer != context)
                || parameter
                    .invalidated_at
                    .is_some_and(|cutoff| offset >= cutoff)
            {
                FlowValue::Unknown
            } else {
                self.parameter(path, parameter, &file)
            }
        } else if let Some(id) = file.resolve_local_callable_id(Some(scope), base) {
            self.callable(path, id)
        } else if scope == 0 {
            let Some(binding) = file.imported.get(base) else {
                self.visiting.remove(&key);
                return FlowValue::Unknown;
            };
            self.imported_named_value(path, &file, base, binding)
        } else {
            FlowValue::Unknown
        };
        if (binding.is_some() || parameter.is_some())
            && context.is_some_and(|context| {
                file.values
                    .receiver_mutations
                    .as_ref()
                    .is_some_and(|receivers| receivers.writer_contexts.contains(&context))
            })
        {
            // A container alias reads live descendants at its use, while
            // acquired member segments retain their existing snapshot nodes.
            self.record_receiver_read(path, offset, context, repeated);
        }
        self.visiting.remove(&key);
        if (scope == 0 && file.imported.contains_key(base))
            || file.values.possible_receiver_mutations.is_some()
        {
            self.record_receiver_origin(path, scope, base, (offset, context, repeated));
        }
        let owned_invocation = self
            .invocation_read
            .as_ref()
            .is_some_and(|read| read.owner.0 == path && source_context == Some(read.owner.1));
        if owned_invocation {
            self.record_invocation_receiver_origin(path, &file, scope, base, (offset, repeated));
        }
        // A member initializer has already checked its source at acquisition.
        // Reading descendants still validates the live receiver projection.
        let acquired_member = Self::acquired_binding_member(binding, member, projection);
        let value = if !acquired_member
            && self.default_origin_invalidated(path, offset, (context, source_context), projection)
        {
            FlowValue::Unknown
        } else {
            value
        };
        // Supplied live receivers acquire an escape guard after source
        // validation. Copied methods keep their independent acquisition phase.
        if !owned_invocation
            && member.is_none()
            && projection.is_none()
            && scope == 0
            && file.imported.contains_key(base)
        {
            self.record_invocation_receiver_origin(path, &file, scope, base, (offset, repeated));
        }
        member.map_or(value.clone(), |member| {
            self.member_at(value, member, path, Some((offset, context)))
        })
    }
}

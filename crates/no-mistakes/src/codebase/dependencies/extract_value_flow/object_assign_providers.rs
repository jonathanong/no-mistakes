use super::*;

impl ImportCollector {
    /// A partial store supplies possible receivers without certifying a fresh
    /// replacement. Reuse the canonical member-origin history, including aliases.
    pub(in super::super::super) fn record_object_assign_providers(
        &mut self,
        callee: &ValueExpression,
        arguments: &[ValueExpression],
        raw: &[oxc_ast::ast::Argument<'_>],
        completed: u32,
    ) {
        let ValueExpression::Member(base, method) = callee else {
            return;
        };
        if method != "assign"
            || !matches!(base.as_ref(), ValueExpression::Named { name, scope: None, .. } if name == "Object")
        {
            return;
        }
        let Some(target) = arguments.first() else {
            return;
        };
        let sources = expanded_assign_sources(self, arguments, raw);
        let mut destinations: Vec<nested::ReceiverWrite> = Vec::new();
        nested::write_receiver_references(target, &[], &[], &mut destinations);
        let mut concrete_writes = Vec::new();
        for ((key, prefix), reads, read) in destinations {
            let version = self.mutable_receiver_version(&key, read);
            let root_identity = version.map(mutable_origins::RootVersion::identity);
            let component = self
                .prior_member_mutations
                .assigned_destination_component(&key);
            for source in &sources {
                // Unknown/source aliases are kept as possible prefix values;
                // literal fields can also participate in ordinary member reads.
                let fields = match source {
                    ValueExpression::Object(fields, _) => fields
                        .iter()
                        .map(|(key, value)| (Some(key.clone()), value))
                        .collect::<Vec<_>>(),
                    _ => vec![(None, source)],
                };
                for (member, value) in fields {
                    let mut path = prefix.clone();
                    let mut acquired = reads.clone();
                    if let Some(member) = member {
                        path.push(Some(member));
                        acquired.push(Some((completed, self.current_function_id())));
                        // A copied field is a member write. Reads must go Unknown
                        // rather than keep the previous known callee.
                        concrete_writes.push((key.0, key.1.clone(), path.clone()));
                    }
                    let mut parent_identities =
                        self.assignment_rhs_parent_identities(&key, &path, &acquired, version);
                    parent_identities.resize(path.len().saturating_sub(1), None);
                    let capture = identity_metadata::FreshCapture {
                        origin: Some(read),
                        reads: self.fresh_receiver_effects.capture(&reads),
                        epoch: self.fresh_receiver_effects.epoch,
                        phase: None,
                        completed_at: completed,
                    };
                    self.prior_member_mutations.record_assigned_origin(
                        component,
                        path,
                        value.clone(),
                        assigned_receiver_origins::Store {
                            owner: self.current_function_id(),
                            reads: &acquired,
                            completed,
                            complete: false,
                            stable_alias: false,
                            capture,
                            root_identity,
                            parent_identities,
                        },
                    );
                }
            }
        }
        for (scope, base, path) in concrete_writes {
            let dotted = std::iter::once(base.as_str())
                .chain(path.iter().filter_map(|part| part.as_deref()))
                .collect::<Vec<_>>()
                .join(".");
            self.invalidate_value_write_with_path(
                scope,
                &dotted,
                completed,
                Some(&path),
                completed,
            );
        }
    }

    /// Literal `Object.assign({}, { client })` is the merged object. Anything
    /// else stays a call so a shadowed or partial assign cannot invent a field.
    pub(in super::super::super) fn object_assign_result(
        &self,
        call: &oxc_ast::ast::CallExpression<'_>,
    ) -> Option<ValueExpression> {
        if simple_callee_name(&call.callee).as_deref() != Some("Object.assign")
            || self.callee_binding_scope("Object").is_some()
        {
            return None;
        }
        let mut raw = call.arguments.iter();
        let target = raw.next()?.as_expression()?;
        let oxc_ast::ast::Expression::ObjectExpression(target) =
            crate::codebase::ts_source::unwrap_ts_wrappers(target)
        else {
            return None;
        };
        let ValueExpression::Object(target_fields, null_prototype) = self.object_value(target)
        else {
            return None;
        };
        let mut fields = (*target_fields).clone();
        for source in raw {
            let expression = source.as_expression()?;
            let oxc_ast::ast::Expression::ObjectExpression(object) =
                crate::codebase::ts_source::unwrap_ts_wrappers(expression)
            else {
                return None;
            };
            let ValueExpression::Object(source_fields, _) = self.object_value(object) else {
                return None;
            };
            for (key, value) in source_fields.iter() {
                fields.insert(key.clone(), value.clone());
            }
        }
        Some(ValueExpression::Object(
            std::sync::Arc::new(fields),
            null_prototype,
        ))
    }
}

fn expanded_assign_sources(
    collector: &ImportCollector,
    arguments: &[ValueExpression],
    raw: &[oxc_ast::ast::Argument<'_>],
) -> Vec<ValueExpression> {
    let mut sources = Vec::new();
    for (index, source) in arguments.iter().enumerate().skip(1) {
        if let Some(oxc_ast::ast::Argument::SpreadElement(spread)) = raw.get(index) {
            let value = collector.value_expression(&spread.argument);
            if let Some(entries) =
                super::super::super::parameter_call_origins::array_entries(collector, &value)
            {
                for (_, value) in entries {
                    sources.push(value);
                }
                continue;
            }
        }
        sources.push(source.clone());
    }
    sources
}

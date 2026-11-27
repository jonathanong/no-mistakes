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
        let sources = super::receiver_escapes::assign_sources(self, arguments, raw, completed);
        let mut destinations: Vec<nested::ReceiverWrite> = Vec::new();
        nested::write_receiver_references(target, &[], &[], &mut destinations);
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
                        super::receiver_escapes::copied_member(self, &key, &path, completed);
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
    }
}

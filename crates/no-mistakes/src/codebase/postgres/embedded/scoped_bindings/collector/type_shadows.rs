use super::ScopeCollector;
use oxc_ast::ast::TSTypeParameterDeclaration;
use oxc_span::Span;

pub(super) type Identity = super::super::super::UndeclaredExecutorType;

pub(super) struct PendingAnnotation {
    pub(super) binding: (String, u32),
    pub(super) candidates: Vec<(String, Vec<String>, Identity)>,
    pub(super) this_scope: Option<Span>,
}

impl ScopeCollector {
    pub(super) fn record_type_shadow(&mut self, name: &str) {
        self.type_shadows
            .push((name.to_string(), *self.scopes.last().expect("scope")));
    }

    pub(super) fn record_type_parameters(
        &mut self,
        parameters: Option<&TSTypeParameterDeclaration<'_>>,
        span: Span,
    ) {
        if let Some(parameters) = parameters {
            self.type_shadows.extend(
                parameters
                    .params
                    .iter()
                    .map(|parameter| (parameter.name.name.to_string(), span)),
            );
        }
    }

    pub(in crate::codebase::postgres::embedded::scoped_bindings) fn finish_undeclared_annotations(
        &mut self,
    ) {
        // Local type declarations shadow throughout their scope, including before
        // their declaration. Resolve after the same walk has seen all declarations.
        for pending in self.pending_annotations.drain(..) {
            let mut identities = super::super::UndeclaredReceiverTypes::new();
            for (path, names, mut identity) in pending.candidates {
                if names.iter().any(|name| {
                    self.type_shadows.iter().any(|(shadow, scope)| {
                        shadow == name
                            && scope.start <= pending.binding.1
                            && pending.binding.1 < scope.end
                    })
                }) {
                    continue;
                }
                identity.explicit_annotation = true;
                identity.known_type = true;
                identity.receiver_name = pending.binding.0.clone();
                identities.entry(path).or_insert(identity);
            }
            if let Some(scope) = pending.this_scope {
                if let Some((_, types)) = self
                    .found
                    .undeclared_this
                    .iter_mut()
                    .find(|(span, _)| *span == scope)
                {
                    types.extend(identities);
                }
            } else {
                self.found
                    .undeclared
                    .entry(pending.binding.1)
                    .or_default()
                    .insert(pending.binding.0, identities);
            }
        }
    }
}

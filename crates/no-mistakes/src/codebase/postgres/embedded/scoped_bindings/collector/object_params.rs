use super::{owners, NameHit, PendingRelativeSpan, ScopeCollector};
use oxc_ast::ast::{BindingPattern, ObjectPattern, TSSignature, TSType};
use oxc_span::{GetSpan, Span};

impl ScopeCollector {
    pub(super) fn record(&mut self, name: &str, span: Span, hit: NameHit) {
        if hit.confirmed {
            self.found.add(name, span);
        } else if !hit.owners.is_empty() {
            self.spans.push(PendingRelativeSpan {
                owners: hit.owners,
                name: name.to_string(),
                start: span.start,
                end: span.end,
            });
        }
    }

    pub(super) fn bind_params(&mut self, params: &oxc_ast::ast::FormalParameters<'_>, scope: Span) {
        for param in &params.items {
            let Some(annotation) = &param.type_annotation else {
                continue;
            };
            let annotated = &annotation.type_annotation;
            match &param.pattern {
                BindingPattern::BindingIdentifier(ident) => {
                    self.record_undeclared(
                        ident.name.as_str(),
                        param.pattern.span().start,
                        annotated,
                    );
                    let hit = owners::type_hit(&self.types, &self.provisional_types, annotated);
                    self.record(
                        ident.name.as_str(),
                        Span::new(param.pattern.span().start, scope.end),
                        hit,
                    );
                }
                BindingPattern::ObjectPattern(object) => {
                    self.bind_object_properties(
                        object,
                        annotated,
                        Span::new(param.pattern.span().start, scope.end),
                    );
                }
                _ => {}
            }
        }
    }

    pub(super) fn bind_object_properties(
        &mut self,
        object: &ObjectPattern<'_>,
        annotated: &TSType<'_>,
        scope: Span,
    ) {
        let TSType::TSTypeLiteral(literal) = annotated else {
            return;
        };
        for property in &object.properties {
            let (Some(key), BindingPattern::BindingIdentifier(local)) =
                (property.key.static_name(), &property.value)
            else {
                continue;
            };
            let mut hit = NameHit {
                confirmed: false,
                owners: Vec::new(),
            };
            let typed = literal.members.iter().any(|member| {
                let TSSignature::TSPropertySignature(signature) = member else {
                    return false;
                };
                if signature.key.static_name().as_deref() != Some(key.as_ref()) {
                    return false;
                }
                let Some(annotation) = &signature.type_annotation else {
                    return false;
                };
                self.record_undeclared(
                    local.name.as_str(),
                    scope.start,
                    &annotation.type_annotation,
                );
                hit = owners::type_hit(
                    &self.types,
                    &self.provisional_types,
                    &annotation.type_annotation,
                );
                hit.confirmed || !hit.owners.is_empty()
            });
            if typed {
                self.record(local.name.as_str(), scope, hit);
            }
        }
    }
}

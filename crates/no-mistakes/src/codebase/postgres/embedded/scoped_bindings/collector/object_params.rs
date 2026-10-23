use super::{owners, NameHit, ScopeCollector};
use oxc_ast::ast::{BindingPattern, ObjectPattern, TSSignature, TSType};
use oxc_span::Span;

impl ScopeCollector {
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

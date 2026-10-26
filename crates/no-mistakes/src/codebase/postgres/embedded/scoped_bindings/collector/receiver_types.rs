use super::{
    type_shadows::{Identity, PendingAnnotation},
    ScopeCollector,
};
use oxc_ast::ast::{TSSignature, TSType, TSTypeName};
use oxc_span::Span;

impl ScopeCollector {
    pub(super) fn record_path_annotation(
        &mut self,
        name: &str,
        start: u32,
        ty: &TSType<'_>,
        path: &str,
        this_scope: Option<Span>,
    ) {
        let candidates = self.annotation_paths(ty, path);
        if candidates.is_empty() {
            return;
        }
        self.pending_annotations.push(PendingAnnotation {
            binding: (name.to_string(), start),
            candidates,
            this_scope,
        });
    }

    fn annotation_paths(
        &self,
        ty: &TSType<'_>,
        path: &str,
    ) -> Vec<(String, Vec<String>, Identity)> {
        match ty {
            TSType::TSTypeReference(reference) => {
                let TSTypeName::IdentifierReference(name) = &reference.type_name else {
                    return Vec::new();
                };
                if let Some(identity) = self.undeclared.get(name.name.as_str()) {
                    return vec![(
                        path.to_string(),
                        vec![name.name.to_string()],
                        identity.clone(),
                    )];
                }
                if matches!(name.name.as_str(), "Array" | "ReadonlyArray") {
                    let Some(element) = reference
                        .type_arguments
                        .as_ref()
                        .and_then(|arguments| arguments.params.first())
                    else {
                        return Vec::new();
                    };
                    let mut paths = self.annotation_paths(element, &format!("{path}[]"));
                    for (_, names, _) in &mut paths {
                        names.push(name.name.to_string());
                    }
                    return paths;
                }
                Vec::new()
            }
            TSType::TSArrayType(array) => {
                self.annotation_paths(&array.element_type, &format!("{path}[]"))
            }
            TSType::TSUnionType(union) => union
                .types
                .iter()
                .flat_map(|member| self.annotation_paths(member, path))
                .collect(),
            TSType::TSTypeLiteral(literal) => literal
                .members
                .iter()
                .flat_map(|member| {
                    let TSSignature::TSPropertySignature(property) = member else {
                        return Vec::new();
                    };
                    let (Some(key), Some(annotation)) = (
                        crate::ast::property_key_name(&property.key),
                        &property.type_annotation,
                    ) else {
                        return Vec::new();
                    };
                    self.annotation_paths(&annotation.type_annotation, &format!("{path}.{key}"))
                })
                .collect(),
            _ => Vec::new(),
        }
    }
}

use super::*;
use crate::codebase::ts_source::module_facts::TypeScriptModuleFacts;
use oxc_ast::ast::{FormalParameters, TSSignature, TSType, TSTypeName};

// Match both type references and parameters by semantic identity. A local type
// with the same spelling as the configured import is never an executor proof.
pub(super) fn collect(
    program: &Program<'_>,
    module: &TypeScriptModuleFacts,
    profile: &EmbeddedSqlOptions,
) -> FxHashSet<usize> {
    let types = module
        .imports
        .iter()
        .filter(|import| {
            super::super::embedded::scoped_bindings::from_configured_module(
                &import.specifier,
                &profile.import_specifier,
            )
        })
        .flat_map(|import| &import.bindings)
        .filter(|binding| profile.executor_type_names.contains(&binding.imported))
        .map(|binding| binding.binding_id)
        .collect::<FxHashSet<_>>();
    let references = module
        .bindings
        .iter()
        .filter(|binding| types.contains(&binding.id))
        .flat_map(|binding| &binding.references)
        .map(|reference| (reference.span.start, reference.span.end))
        .collect();
    let bindings = module
        .bindings
        .iter()
        .map(|binding| ((binding.span.start, binding.span.end), binding.id))
        .collect();
    let mut visitor = Typed {
        references,
        bindings,
        selected: FxHashSet::default(),
    };
    visitor.visit_program(program);
    visitor.selected
}

struct Typed {
    references: FxHashSet<Span>,
    bindings: FxHashMap<Span, usize>,
    selected: FxHashSet<usize>,
}
impl<'a> Visit<'a> for Typed {
    fn visit_formal_parameters(&mut self, parameters: &FormalParameters<'a>) {
        for parameter in &parameters.items {
            let Some(annotation) = &parameter.type_annotation else {
                continue;
            };
            match (&parameter.pattern, &annotation.type_annotation) {
                (BindingPattern::BindingIdentifier(binding), ty) => self.record(binding, ty),
                (BindingPattern::ObjectPattern(object), TSType::TSTypeLiteral(literal)) => {
                    for property in &object.properties {
                        let (Some(key), BindingPattern::BindingIdentifier(binding)) =
                            (property.key.static_name(), &property.value)
                        else {
                            continue;
                        };
                        for signature in &literal.members {
                            if let TSSignature::TSPropertySignature(signature) = signature {
                                if signature.key.static_name().as_deref() == Some(key.as_ref()) {
                                    if let Some(annotation) = &signature.type_annotation {
                                        self.record(binding, &annotation.type_annotation);
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        walk::walk_formal_parameters(self, parameters);
    }
}

impl Typed {
    fn record(&mut self, binding: &oxc_ast::ast::BindingIdentifier<'_>, ty: &TSType<'_>) {
        let TSType::TSTypeReference(reference) = ty else {
            return;
        };
        let TSTypeName::IdentifierReference(name) = &reference.type_name else {
            return;
        };
        if self.references.contains(&span(name.span)) {
            if let Some(id) = self.bindings.get(&span(binding.span)) {
                self.selected.insert(*id);
            }
        }
    }
}

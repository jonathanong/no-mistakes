use super::*;
use crate::codebase::ts_source::module_facts::TypeScriptModuleFacts;
use oxc_ast::ast::{FormalParameters, TSSignature, TSType, TSTypeName};

mod patterns;
mod possible;

// Match both type references and parameters by semantic identity. A local type
// with the same spelling as the configured import is never an executor proof.
pub(super) fn collect(
    program: &Program<'_>,
    module: &TypeScriptModuleFacts,
    profile: &EmbeddedSqlOptions,
    unsupported: &FxHashSet<usize>,
) -> (FxHashSet<usize>, FxHashSet<usize>) {
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
    let candidates = types.union(unsupported).copied().collect();
    let possible = possible::references(program, module, &candidates, &bindings);
    let mut visitor = Typed {
        references,
        possible,
        bindings,
        selected: FxHashSet::default(),
        unsupported: FxHashSet::default(),
    };
    visitor.visit_program(program);
    (visitor.selected, visitor.unsupported)
}

struct Typed {
    references: FxHashSet<Span>,
    possible: FxHashSet<Span>,
    bindings: FxHashMap<Span, usize>,
    selected: FxHashSet<usize>,
    unsupported: FxHashSet<usize>,
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
                        let Some(key) = property.key.static_name() else {
                            continue;
                        };
                        for signature in &literal.members {
                            if let TSSignature::TSPropertySignature(signature) = signature {
                                if signature.key.static_name().as_deref() == Some(key.as_ref()) {
                                    if let Some(annotation) = &signature.type_annotation {
                                        match &property.value {
                                            BindingPattern::BindingIdentifier(binding) => {
                                                self.record(binding, &annotation.type_annotation)
                                            }
                                            pattern => self.unknown_pattern(
                                                pattern,
                                                &annotation.type_annotation,
                                            ),
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if let Some(rest) = &object.rest {
                        self.unknown_pattern(&rest.argument, &annotation.type_annotation);
                    }
                }
                (pattern, ty) => self.unknown_pattern(pattern, ty),
            }
        }
        if let Some(rest) = &parameters.rest {
            if let Some(annotation) = &rest.type_annotation {
                self.unknown_pattern(&rest.rest.argument, &annotation.type_annotation);
            }
        }
        walk::walk_formal_parameters(self, parameters);
    }
}

impl Typed {
    fn record(&mut self, binding: &oxc_ast::ast::BindingIdentifier<'_>, ty: &TSType<'_>) {
        if let Some(id) = self.bindings.get(&span(binding.span)) {
            let direct = match ty {
                TSType::TSTypeReference(reference) => match &reference.type_name {
                    TSTypeName::IdentifierReference(name) => {
                        self.references.contains(&span(name.span))
                    }
                    _ => false,
                },
                _ => false,
            };
            if direct || possible::contains(&self.possible, ty) {
                self.selected.insert(*id);
                if !direct {
                    self.unsupported.insert(*id);
                }
            }
        }
    }
}

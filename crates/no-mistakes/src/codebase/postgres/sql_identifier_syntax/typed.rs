use super::*;
use crate::codebase::ts_source::module_facts::TypeScriptModuleFacts;
use crate::config::v2::schema::EffectTargetConfig;
use oxc_ast::ast::{TSType, TSTypeName};

mod parameters;
mod patterns;
mod possible;
mod runtime;
mod transports;

// Match both type references and parameters by semantic identity. A local type
// with the same spelling as the configured import is never an executor proof.
pub(super) fn collect(
    program: &Program<'_>,
    module: &TypeScriptModuleFacts,
    profile: &EmbeddedSqlOptions,
    selection: &Selection,
    declared_types: &[EffectTargetConfig],
) -> (
    FxHashSet<usize>,
    FxHashSet<usize>,
    FxHashSet<Span>,
    FxHashSet<Span>,
) {
    let declared = declared_types
        .iter()
        .map(|selector| (selector.module.as_str(), selector.export.as_str()))
        .collect::<FxHashSet<_>>();
    let declared = &declared;
    let types = module
        .imports
        .iter()
        .flat_map(|import| {
            import
                .bindings
                .iter()
                .filter(move |binding| {
                    profile.executor_type_names.contains(&binding.imported)
                        && (super::super::embedded::scoped_bindings::from_configured_module(
                            &import.specifier,
                            &profile.import_specifier,
                        ) || declared
                            .contains(&(import.specifier.as_str(), binding.imported.as_str())))
                })
                .map(|binding| binding.binding_id)
        })
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
        .flat_map(|binding| {
            std::iter::once(((binding.span.start, binding.span.end), binding.id)).chain(
                binding
                    .references
                    .iter()
                    .map(|usage| ((usage.span.start, usage.span.end), binding.id)),
            )
        })
        .collect();
    let candidates = types.union(&selection.unsupported).copied().collect();
    let possible = possible::references(program, module, &candidates, &bindings);
    let mut visitor = Typed {
        references,
        possible,
        bindings,
        selected: FxHashSet::default(),
        unsupported: FxHashSet::default(),
        sources: transports::sources(module, selection),
        transports: FxHashSet::default(),
        factory_bindings: FxHashMap::default(),
        writes: FxHashSet::default(),
        immutable: false,
    };
    visitor.visit_program(program);
    visitor.finish_transports();
    let runtime = runtime::collect(program, &visitor.possible);
    (
        visitor.selected,
        visitor.unsupported,
        runtime,
        visitor.transports,
    )
}

struct Typed {
    references: FxHashSet<Span>,
    possible: FxHashSet<Span>,
    bindings: FxHashMap<Span, usize>,
    selected: FxHashSet<usize>,
    unsupported: FxHashSet<usize>,
    sources: FxHashMap<Span, bool>,
    transports: FxHashSet<Span>,
    factory_bindings: FxHashMap<usize, bool>,
    writes: FxHashSet<usize>,
    immutable: bool,
}
impl<'a> Visit<'a> for Typed {
    fn visit_variable_declarator(&mut self, variable: &oxc_ast::ast::VariableDeclarator<'a>) {
        if let Some(annotation) = &variable.type_annotation {
            self.unknown_pattern(&variable.id, &annotation.type_annotation);
        }
        self.factory_binding(variable);
        walk::walk_variable_declarator(self, variable);
    }
    fn visit_formal_parameters(&mut self, parameters: &oxc_ast::ast::FormalParameters<'a>) {
        self.parameters(parameters);
        walk::walk_formal_parameters(self, parameters);
    }
    fn visit_variable_declaration(&mut self, value: &VariableDeclaration<'a>) {
        let old = self.immutable;
        self.immutable = matches!(
            value.kind,
            oxc_ast::ast::VariableDeclarationKind::Const
                | oxc_ast::ast::VariableDeclarationKind::Using
                | oxc_ast::ast::VariableDeclarationKind::AwaitUsing
        );
        walk::walk_variable_declaration(self, value);
        self.immutable = old;
    }
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        self.wrapper(call);
        walk::walk_call_expression(self, call);
    }
    fn visit_simple_assignment_target(
        &mut self,
        target: &oxc_ast::ast::SimpleAssignmentTarget<'a>,
    ) {
        if let Some(at) = super::template_binds::assignment_root(target) {
            self.write(at);
        }
        if let oxc_ast::ast::SimpleAssignmentTarget::ComputedMemberExpression(value) = target {
            if let Some((root, _)) = chain(&value.expression) {
                self.transports.insert(root);
            }
        }
        walk::walk_simple_assignment_target(self, target);
    }
    fn visit_assignment_target_property_identifier(
        &mut self,
        target: &oxc_ast::ast::AssignmentTargetPropertyIdentifier<'a>,
    ) {
        self.write(span(target.binding.span));
        walk::walk_assignment_target_property_identifier(self, target);
    }
}

impl Typed {
    fn direct_type(&self, ty: &TSType<'_>) -> bool {
        matches!(ty, TSType::TSTypeReference(reference)
            if matches!(&reference.type_name, TSTypeName::IdentifierReference(name)
                if self.references.contains(&span(name.span))))
    }
    // Only direct imported types on formal parameters prove literal SQL. Other
    // annotation shapes retain runtime Unknown uses without evaluating SQL.
    fn record(&mut self, binding: &oxc_ast::ast::BindingIdentifier<'_>, ty: &TSType<'_>) {
        if let Some(id) = self.bindings.get(&span(binding.span)) {
            let direct = self.direct_type(ty);
            if direct || possible::contains(&self.possible, ty) {
                self.selected.insert(*id);
                if !direct {
                    self.unsupported.insert(*id);
                }
            }
        }
    }
}

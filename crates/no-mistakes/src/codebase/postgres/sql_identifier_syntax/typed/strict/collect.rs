use super::*;
use oxc_ast::ast::{
    FormalParameters, SimpleAssignmentTarget, TSTypeAliasDeclaration, VariableDeclarator,
};

pub(super) struct Facts<'b> {
    bindings: &'b FxHashMap<Span, usize>,
    pub aliases: Vec<(usize, Shape)>,
    pub parameters: Vec<(usize, Shape)>,
    pub initializers: Vec<(usize, usize)>,
    pub constants: Vec<(usize, usize)>,
    pub annotations: Vec<(Span, Shape)>,
    pub writes: FxHashSet<usize>,
    immutable: bool,
}
impl<'b> Facts<'b> {
    pub fn new(bindings: &'b FxHashMap<Span, usize>) -> Self {
        Self {
            bindings,
            aliases: Vec::new(),
            parameters: Vec::new(),
            constants: Vec::new(),
            initializers: Vec::new(),
            annotations: Vec::new(),
            writes: FxHashSet::default(),
            immutable: false,
        }
    }
}
impl<'a> Visit<'a> for Facts<'_> {
    fn visit_ts_type(&mut self, ty: &TSType<'a>) {
        self.annotations.push((
            span(ty.span()),
            Lower {
                bindings: self.bindings,
            }
            .shape(ty),
        ));
        walk::walk_ts_type(self, ty);
    }
    fn visit_ts_type_alias_declaration(&mut self, alias: &TSTypeAliasDeclaration<'a>) {
        if alias.type_parameters.is_none() {
            if let Some(id) = self.bindings.get(&span(alias.id.span)) {
                self.aliases.push((
                    *id,
                    Lower {
                        bindings: self.bindings,
                    }
                    .shape(&alias.type_annotation),
                ));
            }
        }
        walk::walk_ts_type_alias_declaration(self, alias);
    }
    fn visit_formal_parameters(&mut self, parameters: &FormalParameters<'a>) {
        for parameter in &parameters.items {
            if parameter.initializer.is_none() {
                if let (BindingPattern::BindingIdentifier(binding), Some(ty)) =
                    (&parameter.pattern, &parameter.type_annotation)
                {
                    if let Some(id) = self.bindings.get(&span(binding.span)) {
                        self.parameters.push((
                            *id,
                            Lower {
                                bindings: self.bindings,
                            }
                            .shape(&ty.type_annotation),
                        ));
                    }
                }
            }
        }
        walk::walk_formal_parameters(self, parameters);
    }
    fn visit_variable_declaration(&mut self, declaration: &VariableDeclaration<'a>) {
        let previous = self.immutable;
        self.immutable = matches!(
            declaration.kind,
            oxc_ast::ast::VariableDeclarationKind::Const
                | oxc_ast::ast::VariableDeclarationKind::Using
                | oxc_ast::ast::VariableDeclarationKind::AwaitUsing
        );
        walk::walk_variable_declaration(self, declaration);
        self.immutable = previous;
    }
    fn visit_variable_declarator(&mut self, declaration: &VariableDeclarator<'a>) {
        if let (BindingPattern::BindingIdentifier(binding), Some(init)) =
            (&declaration.id, &declaration.init)
        {
            if let Some(id) = self.bindings.get(&span(binding.span)) {
                if let Some(factory) = (Lower {
                    bindings: self.bindings,
                })
                .factory_call(init)
                {
                    self.initializers.push((*id, factory));
                    if self.immutable {
                        self.constants.push((*id, factory));
                    }
                }
            }
        }
        walk::walk_variable_declarator(self, declaration);
    }
    fn visit_simple_assignment_target(&mut self, target: &SimpleAssignmentTarget<'a>) {
        if let SimpleAssignmentTarget::AssignmentTargetIdentifier(name) = target {
            if let Some(id) = self.bindings.get(&span(name.span)) {
                self.writes.insert(*id);
            }
        }
        walk::walk_simple_assignment_target(self, target);
    }
    fn visit_assignment_target_property_identifier(
        &mut self,
        target: &oxc_ast::ast::AssignmentTargetPropertyIdentifier<'a>,
    ) {
        if let Some(id) = self.bindings.get(&span(target.binding.span)) {
            self.writes.insert(*id);
        }
        walk::walk_assignment_target_property_identifier(self, target);
    }
}

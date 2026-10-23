mod class_receivers;
mod object_params;
mod receiver_types;
mod type_shadows;
mod undeclared;
use super::super::relative::PendingRelativeSpan;
use super::candidates::ImportClassification;
use super::owners::{self, NameHit};
use super::ScopedExecutors;
use oxc_ast::ast::{
    ArrowFunctionExpression, BindingPattern, BlockStatement, ForStatement, Function, FunctionBody,
    Program, StaticBlock, SwitchStatement, VariableDeclaration, VariableDeclarationKind,
};
use oxc_ast_visit::{walk, Visit};
use oxc_span::{GetSpan, Span};
use oxc_syntax::scope::ScopeFlags;
use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub(super) struct ScopeCollector {
    factories: HashSet<String>,
    types: HashSet<String>,
    undeclared: HashMap<String, super::super::UndeclaredExecutorType>,
    provisional_factories: HashMap<String, Vec<u32>>,
    provisional_types: HashMap<String, Vec<u32>>,
    scopes: Vec<Span>,
    type_shadows: Vec<(String, Span)>,
    classes: Vec<Span>,
    class_methods: HashSet<u32>,
    pending_annotations: Vec<type_shadows::PendingAnnotation>,
    pub(super) found: ScopedExecutors,
    pub(super) spans: Vec<PendingRelativeSpan>,
}

impl ScopeCollector {
    pub(super) fn needs_walk(&self) -> bool {
        !self.undeclared.is_empty()
            || !self.factories.is_empty()
            || !self.types.is_empty()
            || !self.provisional_factories.is_empty()
            || !self.provisional_types.is_empty()
    }

    fn record(&mut self, name: &str, span: Span, hit: NameHit) {
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

    fn bind_params(&mut self, params: &oxc_ast::ast::FormalParameters<'_>, scope: Span) {
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

    fn with_scope(&mut self, span: Span, walk: impl FnOnce(&mut Self)) {
        self.scopes.push(span);
        walk(self);
        self.scopes.pop();
    }
}

impl<'a> Visit<'a> for ScopeCollector {
    fn visit_program(&mut self, program: &Program<'a>) {
        self.with_scope(program.span, |this| walk::walk_program(this, program));
    }

    fn visit_block_statement(&mut self, block: &BlockStatement<'a>) {
        self.with_scope(block.span, |this| walk::walk_block_statement(this, block));
    }

    fn visit_function_body(&mut self, body: &FunctionBody<'a>) {
        self.with_scope(body.span, |this| walk::walk_function_body(this, body));
    }

    fn visit_static_block(&mut self, block: &StaticBlock<'a>) {
        self.found.undeclared_this_exclusions.push(block.span);
        self.with_scope(block.span, |this| walk::walk_static_block(this, block));
    }

    fn visit_switch_statement(&mut self, switch: &SwitchStatement<'a>) {
        self.with_scope(switch.span, |this| {
            walk::walk_switch_statement(this, switch)
        });
    }

    fn visit_for_statement(&mut self, statement: &ForStatement<'a>) {
        self.with_scope(statement.span, |this| {
            walk::walk_for_statement(this, statement)
        });
    }

    fn visit_function(&mut self, function: &Function<'a>, flags: ScopeFlags) {
        self.exclude_dynamic_this(function);
        self.record_type_parameters(function.type_parameters.as_deref(), function.span);
        self.bind_params(&function.params, function.span);
        walk::walk_function(self, function, flags);
    }

    fn visit_arrow_function_expression(&mut self, arrow: &ArrowFunctionExpression<'a>) {
        self.record_type_parameters(arrow.type_parameters.as_deref(), arrow.span);
        self.bind_params(&arrow.params, arrow.span);
        walk::walk_arrow_function_expression(self, arrow);
    }

    fn visit_class(&mut self, class: &oxc_ast::ast::Class<'a>) {
        if let Some(name) = &class.id {
            let span = if class.r#type == oxc_ast::ast::ClassType::ClassDeclaration {
                *self.scopes.last().expect("scope")
            } else {
                class.span
            };
            self.type_shadows.push((name.name.to_string(), span));
        }
        self.record_type_parameters(class.type_parameters.as_deref(), class.span);
        self.prepare_class_receivers(class);
        self.classes.push(class.span);
        walk::walk_class(self, class);
        self.classes.pop();
    }

    fn visit_property_definition(&mut self, property: &oxc_ast::ast::PropertyDefinition<'a>) {
        self.record_class_property(property);
        walk::walk_property_definition(self, property);
    }

    fn visit_formal_parameter(&mut self, parameter: &oxc_ast::ast::FormalParameter<'a>) {
        self.record_parameter_property(parameter);
        walk::walk_formal_parameter(self, parameter);
    }

    fn visit_ts_type_alias_declaration(
        &mut self,
        alias: &oxc_ast::ast::TSTypeAliasDeclaration<'a>,
    ) {
        self.record_type_shadow(alias.id.name.as_str());
        walk::walk_ts_type_alias_declaration(self, alias);
    }

    fn visit_ts_interface_declaration(
        &mut self,
        interface: &oxc_ast::ast::TSInterfaceDeclaration<'a>,
    ) {
        self.record_type_shadow(interface.id.name.as_str());
        walk::walk_ts_interface_declaration(self, interface);
    }

    fn visit_variable_declarator(&mut self, declaration: &oxc_ast::ast::VariableDeclarator<'a>) {
        self.record_undeclared_variable(declaration);
        walk::walk_variable_declarator(self, declaration);
    }

    fn visit_variable_declaration(&mut self, declaration: &VariableDeclaration<'a>) {
        // `var` hoists past the block, so only block-scoped kinds are bound.
        if declaration.kind != VariableDeclarationKind::Var {
            let end = self.scopes.last().expect("program scope").end;
            for declarator in &declaration.declarations {
                if let (BindingPattern::BindingIdentifier(ident), Some(init)) =
                    (&declarator.id, &declarator.init)
                {
                    let hit =
                        owners::factory_hit(&self.factories, &self.provisional_factories, init);
                    let span = Span::new(declarator.id.span().start, end);
                    self.record(ident.name.as_str(), span, hit);
                }
            }
        }
        walk::walk_variable_declaration(self, declaration);
    }
}

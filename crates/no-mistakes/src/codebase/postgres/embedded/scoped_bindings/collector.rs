use super::super::relative::PendingRelativeSpan;
use super::candidates::ImportClassification;
use super::owners::{self, NameHit};
use super::ScopedExecutors;
use oxc_ast::ast::{
    ArrowFunctionExpression, BindingPattern, BlockStatement, ForStatement, Function, FunctionBody,
    ObjectPattern, Program, StaticBlock, SwitchStatement, TSSignature, TSType, VariableDeclaration,
    VariableDeclarationKind,
};
use oxc_ast_visit::{walk, Visit};
use oxc_span::{GetSpan, Span};
use oxc_syntax::scope::ScopeFlags;
use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub(super) struct ScopeCollector {
    factories: HashSet<String>,
    types: HashSet<String>,
    provisional_factories: HashMap<String, Vec<u32>>,
    provisional_types: HashMap<String, Vec<u32>>,
    scopes: Vec<Span>,
    pub(super) found: ScopedExecutors,
    pub(super) spans: Vec<PendingRelativeSpan>,
}

impl ScopeCollector {
    pub(super) fn from_imports(imports: ImportClassification) -> Self {
        Self {
            factories: imports.factories,
            types: imports.types,
            provisional_factories: imports.provisional_factories,
            provisional_types: imports.provisional_types,
            ..Self::default()
        }
    }

    pub(super) fn needs_walk(&self) -> bool {
        !self.factories.is_empty()
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
                    let hit = owners::type_hit(&self.types, &self.provisional_types, annotated);
                    self.record(ident.name.as_str(), scope, hit);
                }
                BindingPattern::ObjectPattern(object) => {
                    self.bind_object_properties(object, annotated, scope);
                }
                _ => {}
            }
        }
    }

    fn bind_object_properties(
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
        self.bind_params(&function.params, function.span);
        walk::walk_function(self, function, flags);
    }

    fn visit_arrow_function_expression(&mut self, arrow: &ArrowFunctionExpression<'a>) {
        self.bind_params(&arrow.params, arrow.span);
        walk::walk_arrow_function_expression(self, arrow);
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
                    let span = Span::new(declarator.span().start, end);
                    self.record(ident.name.as_str(), span, hit);
                }
            }
        }
        walk::walk_variable_declaration(self, declaration);
    }
}

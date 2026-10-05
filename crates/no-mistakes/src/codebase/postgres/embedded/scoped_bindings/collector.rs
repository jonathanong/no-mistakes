use super::ScopedExecutors;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{
    ArrowFunctionExpression, BindingPattern, BlockStatement, Expression, ForStatement, Function,
    FunctionBody, ObjectPattern, Program, StaticBlock, SwitchStatement, TSSignature, TSType,
    TSTypeName, VariableDeclaration, VariableDeclarationKind,
};
use oxc_ast_visit::{walk, Visit};
use oxc_span::{GetSpan, Span};
use oxc_syntax::scope::ScopeFlags;
use std::collections::HashSet;

#[derive(Default)]
pub(super) struct ScopeCollector {
    pub(super) factories: HashSet<String>,
    pub(super) types: HashSet<String>,
    scopes: Vec<Span>,
    pub(super) found: ScopedExecutors,
}

impl ScopeCollector {
    fn type_matches(&self, ty: &TSType<'_>) -> bool {
        match ty {
            TSType::TSTypeReference(reference) => matches!(
                &reference.type_name,
                TSTypeName::IdentifierReference(ident) if self.types.contains(ident.name.as_str())
            ),
            TSType::TSUnionType(union) => {
                union.types.iter().any(|member| self.type_matches(member))
            }
            _ => false,
        }
    }

    fn is_factory_call(&self, init: &Expression<'_>) -> bool {
        let mut init = unwrap_ts_wrappers(init);
        if let Expression::AwaitExpression(awaited) = init {
            init = unwrap_ts_wrappers(&awaited.argument);
        }
        let Expression::CallExpression(call) = init else {
            return false;
        };
        matches!(
            unwrap_ts_wrappers(&call.callee),
            Expression::Identifier(ident) if self.factories.contains(ident.name.as_str())
        )
    }

    fn bind_params(&mut self, params: &oxc_ast::ast::FormalParameters<'_>, scope: Span) {
        for param in &params.items {
            let Some(annotation) = &param.type_annotation else {
                continue;
            };
            let annotated = &annotation.type_annotation;
            match &param.pattern {
                BindingPattern::BindingIdentifier(ident) if self.type_matches(annotated) => {
                    self.found.add(ident.name.as_str(), scope);
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
            let typed = literal.members.iter().any(|member| {
                let TSSignature::TSPropertySignature(signature) = member else {
                    return false;
                };
                signature.key.static_name().as_deref() == Some(key.as_ref())
                    && signature
                        .type_annotation
                        .as_ref()
                        .is_some_and(|annotation| self.type_matches(&annotation.type_annotation))
            });
            if typed {
                self.found.add(local.name.as_str(), scope);
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
                    if self.is_factory_call(init) {
                        let span = Span::new(declarator.span().start, end);
                        self.found.add(ident.name.as_str(), span);
                    }
                }
            }
        }
        walk::walk_variable_declaration(self, declaration);
    }
}

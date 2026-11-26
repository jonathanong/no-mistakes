use super::TypeScriptModuleFacts;
use crate::fx::FxHashMap;
use oxc_ast::{ast::*, AstKind};
use oxc_ast_visit::Visit;
use oxc_semantic::Semantic;
use oxc_syntax::scope::ScopeFlags;

pub(super) fn collect(semantic: &Semantic<'_>, facts: &mut TypeScriptModuleFacts) {
    let scoping = semantic.scoping();
    let mut checked = FxHashMap::default();
    facts.receiver_safe_exports.extend(
        facts
            .exports
            .iter()
            .filter(|export| export.specifier.is_none() && !export.type_only)
            .filter(|export| {
                let Some(symbol) = scoping.get_binding(
                    scoping.root_scope_id(),
                    oxc_str::Ident::from(export.local.as_str()),
                ) else {
                    return false;
                };
                *checked.entry(symbol).or_insert_with(|| {
                    if scoping.symbol_flags(symbol).is_ambient()
                        || !scoping.symbol_redeclarations(symbol).is_empty()
                        || scoping
                            .get_resolved_references(symbol)
                            .any(|reference| reference.is_write())
                    {
                        return false;
                    }
                    match semantic.nodes().kind(scoping.symbol_declaration(symbol)) {
                        AstKind::VariableDeclarator(value) => {
                            matches!(value.id, BindingPattern::BindingIdentifier(_))
                                && matches!(
                                    value.init.as_ref().map(Expression::get_inner_expression),
                                    Some(Expression::ArrowFunctionExpression(_))
                                )
                        }
                        AstKind::Function(value) => function_safe(value),
                        AstKind::Class(value) => class_safe(value),
                        _ => false,
                    }
                })
            })
            .map(|export| export.exported.clone()),
    );
}

pub(super) fn default_safe(value: &ExportDefaultDeclaration<'_>) -> bool {
    match &value.declaration {
        ExportDefaultDeclarationKind::FunctionDeclaration(value) if value.id.is_none() => {
            function_safe(value)
        }
        ExportDefaultDeclarationKind::ClassDeclaration(value) if value.id.is_none() => {
            class_safe(value)
        }
        _ => matches!(
            value
                .declaration
                .as_expression()
                .map(Expression::get_inner_expression),
            Some(Expression::ArrowFunctionExpression(_))
        ),
    }
}

fn function_safe(value: &Function<'_>) -> bool {
    value.body.as_ref().is_some_and(|body| {
        let mut visitor = UsesThis(false);
        visitor.visit_formal_parameters(&value.params);
        visitor.visit_function_body(body);
        !visitor.0
    })
}

fn class_safe(value: &Class<'_>) -> bool {
    let mut visitor = UsesThis(false);
    visitor.visit_class(value);
    !visitor.0
}

struct UsesThis(bool);
impl<'a> Visit<'a> for UsesThis {
    fn visit_this_expression(&mut self, _: &ThisExpression) {
        self.0 = true;
    }

    fn visit_function(&mut self, _: &Function<'a>, _: ScopeFlags) {}
}

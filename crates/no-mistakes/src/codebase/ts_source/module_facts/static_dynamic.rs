use super::{ModuleLoad, TypeScriptModuleFacts};
use crate::codebase::dependencies::extract::static_import_specifier;
use crate::fx::FxHashMap;
use oxc_ast::{ast::*, AstKind};
use oxc_semantic::Semantic;
use oxc_syntax::symbol::SymbolId;

mod url_value;

#[derive(Debug, Clone)]
pub(crate) struct FoldedImport {
    pub(crate) load: ModuleLoad,
    pub(crate) relative_url: bool,
    pub(crate) pathname: bool,
    pub(crate) source: String,
}

#[derive(Clone)]
struct Value {
    specifier: String,
    relative_url: bool,
    pathname: bool,
    object: bool,
}

pub(super) struct Collector<'s, 'a> {
    semantic: &'s Semantic<'a>,
    bindings: FxHashMap<SymbolId, Option<Value>>,
}

impl<'s, 'a> Collector<'s, 'a> {
    pub(super) fn new(semantic: &'s Semantic<'a>) -> Self {
        Self {
            semantic,
            bindings: FxHashMap::default(),
        }
    }

    pub(super) fn collect(
        &mut self,
        import: &ImportExpression<'a>,
        node: oxc_semantic::NodeId,
        facts: &mut TypeScriptModuleFacts,
    ) {
        if static_import_specifier(&import.source).is_some() {
            return;
        }
        let Some(value) = self.value(&import.source, 0) else {
            return;
        };
        if !value.relative_url && value.specifier.contains(':') {
            return;
        }
        if self.discarded(node) {
            facts
                .dynamic_import_selectors
                .push((import.span.start, Vec::new(), Vec::new()));
        }
        let source = match import.source.get_inner_expression() {
            Expression::Identifier(identifier) => identifier.name.to_string(),
            _ => "<computed>".into(),
        };
        facts
            .importer_safe
            .push((import.span.start, import.span.end));
        facts.folded_dynamic_imports.push(FoldedImport {
            load: ModuleLoad {
                kind: "dynamicImport".into(),
                specifier: value.specifier,
                span: import.span.into(),
            },
            relative_url: value.relative_url,
            pathname: value.pathname,
            source,
        });
    }

    fn discarded(&self, node: oxc_semantic::NodeId) -> bool {
        self.semantic
            .nodes()
            .ancestor_kinds(node)
            .find_map(|kind| match kind {
                AstKind::ParenthesizedExpression(_)
                | AstKind::TSAsExpression(_)
                | AstKind::TSSatisfiesExpression(_)
                | AstKind::TSNonNullExpression(_)
                | AstKind::TSInstantiationExpression(_)
                | AstKind::TSTypeAssertion(_)
                | AstKind::AwaitExpression(_) => None,
                AstKind::ExpressionStatement(_) => Some(true),
                AstKind::UnaryExpression(value) => {
                    Some(value.operator == oxc_syntax::operator::UnaryOperator::Void)
                }
                _ => Some(false),
            })
            .unwrap_or(false)
    }

    fn value(&mut self, expression: &Expression<'a>, depth: usize) -> Option<Value> {
        if depth >= 32 {
            return None;
        }
        if let Some(specifier) = static_import_specifier(expression) {
            return Some(Value {
                specifier,
                relative_url: false,
                pathname: false,
                object: false,
            });
        }
        match expression.get_inner_expression() {
            Expression::Identifier(identifier) => self.binding(identifier, depth + 1),
            Expression::NewExpression(created) => self.url(created),
            Expression::StaticMemberExpression(member)
                if !member.optional
                    && matches!(member.property.name.as_str(), "href" | "pathname") =>
            {
                let Expression::NewExpression(created) = member.object.get_inner_expression()
                else {
                    return None;
                };
                let mut value = self.url(created)?;
                if member.property.name == "pathname" && value.specifier.contains(['%', '?', '#']) {
                    return None;
                }
                value.object = false;
                value.pathname = member.property.name == "pathname";
                Some(value)
            }
            Expression::CallExpression(call) if !call.optional && call.arguments.len() == 1 => {
                if !self.global(&call.callee, "String") {
                    return None;
                }
                let Expression::NewExpression(created) =
                    call.arguments[0].as_expression()?.get_inner_expression()
                else {
                    return None;
                };
                let mut value = self.url(created)?;
                value.object = false;
                Some(value)
            }
            _ => None,
        }
    }

    fn binding(&mut self, identifier: &IdentifierReference<'a>, depth: usize) -> Option<Value> {
        let scoping = self.semantic.scoping();
        let symbol = scoping
            .get_reference(identifier.reference_id.get()?)
            .symbol_id()?;
        let AstKind::VariableDeclarator(declaration) = self
            .semantic
            .nodes()
            .kind(scoping.symbol_declaration(symbol))
        else {
            return None;
        };
        if identifier.span.start < declaration.span.end {
            return None;
        }
        if let Some(value) = self.bindings.get(&symbol) {
            return value.clone();
        }
        self.bindings.insert(symbol, None);
        if !scoping.symbol_flags(symbol).is_const_variable()
            || !scoping.symbol_redeclarations(symbol).is_empty()
            || scoping
                .get_resolved_references(symbol)
                .any(|reference| reference.is_write())
        {
            return None;
        }
        if !matches!(declaration.id, BindingPattern::BindingIdentifier(_)) {
            return None;
        }
        let value = self.value(declaration.init.as_ref()?, depth)?;
        if value.object {
            return None;
        }
        self.bindings.insert(symbol, Some(value.clone()));
        Some(value)
    }
}

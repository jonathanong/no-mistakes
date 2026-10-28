//! A source-visible Set or global constructor alias voids local intrinsic trust.
use crate::fx::FxHashSet;
use oxc_ast::ast::{self, Expression, ImportDeclarationSpecifier, ImportOrExportKind};
use oxc_ast_visit::{walk, Visit};
use oxc_syntax::scope::ScopeFlags;

pub(in crate::codebase::postgres::embedded::walk) fn intrinsic_safe(
    program: &ast::Program<'_>,
) -> bool {
    let mut uses = Intrinsics {
        allowed: FxHashSet::default(),
        safe: true,
    };
    uses.visit_program(program);
    uses.safe
}
struct Intrinsics {
    allowed: FxHashSet<u32>,
    safe: bool,
}
impl<'a> Visit<'a> for Intrinsics {
    fn visit_variable_declaration(&mut self, declaration: &ast::VariableDeclaration<'a>) {
        for item in &declaration.declarations {
            super::super::resolve::for_each_bound_name(&item.id, &mut |name| {
                if name == "Set" {
                    self.safe = false;
                }
            });
        }
        walk::walk_variable_declaration(self, declaration);
    }
    fn visit_function(&mut self, function: &ast::Function<'a>, flags: ScopeFlags) {
        if function.r#type == ast::FunctionType::FunctionDeclaration
            && function.id.as_ref().is_some_and(|id| id.name == "Set")
        {
            self.safe = false;
        }
        walk::walk_function(self, function, flags);
    }
    fn visit_class(&mut self, class: &ast::Class<'a>) {
        if class.r#type == ast::ClassType::ClassDeclaration
            && class.id.as_ref().is_some_and(|id| id.name == "Set")
        {
            self.safe = false;
        }
        walk::walk_class(self, class);
    }
    fn visit_import_declaration(&mut self, import: &ast::ImportDeclaration<'a>) {
        if import.import_kind == ImportOrExportKind::Type {
            return;
        }
        for specifier in import.specifiers.iter().flatten() {
            let local = match specifier {
                ImportDeclarationSpecifier::ImportSpecifier(named)
                    if named.import_kind != ImportOrExportKind::Type =>
                {
                    &named.local.name
                }
                ImportDeclarationSpecifier::ImportDefaultSpecifier(default) => &default.local.name,
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(namespace) => {
                    &namespace.local.name
                }
                _ => continue,
            };
            if local == "Set" {
                self.safe = false;
            }
        }
    }
    fn visit_new_expression(&mut self, new: &ast::NewExpression<'a>) {
        if let Expression::Identifier(id) = &new.callee {
            if id.name == "Set" {
                self.allowed.insert(id.span.start);
            }
        }
        walk::walk_new_expression(self, new);
    }
    fn visit_identifier_reference(&mut self, id: &ast::IdentifierReference<'a>) {
        if id.name == "Set" && !self.allowed.contains(&id.span.start)
            || matches!(
                id.name.as_str(),
                "globalThis" | "global" | "window" | "self"
            )
        {
            self.safe = false;
        }
    }
}

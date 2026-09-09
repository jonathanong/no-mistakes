use super::{record_variable_declaration, ScopeVisitor};
use oxc_ast::ast::{ForStatement, ForStatementInit, ForStatementLeft, VariableDeclarationKind};

/// Binds a for-in/for-of declaration-form loop target (`for (const build of
/// providers) {}`) into the scope `visit_for_in_statement`/
/// `visit_for_of_statement` just pushed, as a local shadow — matching how
/// `bind_param` shadows a function parameter. The non-declaration form
/// (`for (build of providers)`, reassigning an existing outer binding) needs
/// no such binding: `ReassignedNames` already drops that name from
/// `LocalFunctions` everywhere, inside the loop and out.
///
/// This runs regardless of `var`/`let`/`const`: whatever the loop body sees
/// while it runs, the per-iteration value shadows a same-named top-level
/// helper. Whether the name counts as reassigned *after* the loop — where
/// only `var` leaks — is a separate question `ReassignedNames::
/// visit_for_statement_left` answers on its own.
pub(crate) fn bind_for_statement_left(left: &ForStatementLeft<'_>, visitor: &mut ScopeVisitor<'_>) {
    if let ForStatementLeft::VariableDeclaration(declaration) = left {
        for declarator in &declaration.declarations {
            visitor.bind_param(&declarator.id);
        }
    }
}

/// `var` classic-for initializers are function-scoped, so they must be
/// recorded in the enclosing scope — a loop scope would drop them on exit
/// and fail-closed `query(q)` after `for (var q = "SELECT 1"; false;) {}`.
pub(crate) fn enter_classic_for(statement: &ForStatement<'_>, visitor: &mut ScopeVisitor<'_>) {
    if classic_for_init_is_var(statement) {
        if let Some(ForStatementInit::VariableDeclaration(declaration)) = &statement.init {
            record_variable_declaration(declaration, visitor);
        }
        return;
    }
    visitor.push_scope();
    if let Some(ForStatementInit::VariableDeclaration(declaration)) = &statement.init {
        for declarator in &declaration.declarations {
            visitor.bind_param(&declarator.id);
        }
    }
}

pub(crate) fn leave_classic_for(statement: &ForStatement<'_>, visitor: &mut ScopeVisitor<'_>) {
    if !classic_for_init_is_var(statement) {
        visitor.pop_scope();
    }
}

fn classic_for_init_is_var(statement: &ForStatement<'_>) -> bool {
    matches!(
        &statement.init,
        Some(ForStatementInit::VariableDeclaration(declaration))
            if declaration.kind == VariableDeclarationKind::Var
    )
}

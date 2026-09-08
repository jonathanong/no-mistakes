use super::super::for_each_bound_name;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{
    BindingPattern, Declaration, Expression, ImportDeclaration, ImportDeclarationSpecifier,
    ImportOrExportKind, Program, Statement, VariableDeclaration, VariableDeclarator,
};
use std::collections::HashSet;

/// Top-level `const`/`let`/`var` bindings whose initializer isn't itself a
/// same-file callable helper shape (`function`/arrow expression) — anything
/// else rebinds the name away from whatever a same-file helper's body would
/// otherwise trust it to mean when referencing it through closure (not
/// through the helper's own parameters, which `shadows_param` already
/// covers). A top-level `function` declaration is out of scope here: that
/// shape is a legitimate same-file helper collected elsewhere, including one
/// referencing its own top-level declaration by name.
///
/// The callable-helper-shape exemption never applies to a binding spelled
/// `sql` (case-insensitively, matching the tag-name check this feeds): the
/// only thing that consults a shadowed name is whether it is safe to trust a
/// tagged template's own tag as the trusted SQL-concatenation tag, and a
/// callable rebinding of `sql` is exactly the shape that can ignore its
/// template arguments and return arbitrary text — a helper shape doesn't
/// make that any safer.
#[derive(Default)]
pub(super) struct TagShadows {
    names: HashSet<String>,
}

impl TagShadows {
    pub(super) fn collect(program: &Program<'_>) -> Self {
        let mut shadows = Self::default();
        let top_level_functions = top_level_function_names(program);
        for statement in &program.body {
            record_statement(statement, &top_level_functions, &mut shadows);
        }
        shadows
    }

    pub(super) fn contains(&self, name: &str) -> bool {
        self.names.contains(name)
    }
}

/// Names of every top-level `function` declaration (including
/// `export function …`), gathered up front so a same-named top-level
/// `const`/`let`/`var` never counts as shadowing it — matching the
/// established rule in [`super::super::record_function_declaration`]'s own
/// doc comment: "existing fixtures rely on a same-named top-level helper
/// never shadowing itself." The parser doesn't reject the real-world
/// collision (JS would), and fixtures such as
/// `composed-chain-append-tagged-trusted.ts` and `composed-concat-tagged.ts`
/// deliberately reuse the trusted tag's own name for a const holding its
/// composed result.
fn top_level_function_names<'a>(program: &Program<'a>) -> HashSet<&'a str> {
    let mut names = HashSet::new();
    for statement in &program.body {
        let function = match statement {
            Statement::FunctionDeclaration(function) => Some(function.as_ref()),
            Statement::ExportDeclaration(export) => match &export.declaration {
                Declaration::FunctionDeclaration(function) => Some(function.as_ref()),
                _ => None,
            },
            _ => None,
        };
        if let Some(id) = function.and_then(|function| function.id.as_ref()) {
            names.insert(id.name.as_str());
        }
    }
    names
}

fn record_statement(
    statement: &Statement<'_>,
    top_level_functions: &HashSet<&str>,
    shadows: &mut TagShadows,
) {
    match statement {
        Statement::VariableDeclaration(declaration) => {
            record_declaration(declaration, top_level_functions, shadows);
        }
        Statement::ExportDeclaration(export) => {
            if let Declaration::VariableDeclaration(declaration) = &export.declaration {
                record_declaration(declaration, top_level_functions, shadows);
            }
        }
        Statement::ImportDeclaration(import) => record_import(import, shadows),
        _ => {}
    }
}

/// An imported local binding spelled `sql` is exactly as untrusted as a
/// non-helper-shaped top-level rebinding: nothing here verifies the
/// import's source module actually is the trusted SQL-concatenation tag, so
/// a same-spelled import from anywhere else can ignore its template
/// argument and return arbitrary text.
fn record_import(import: &ImportDeclaration<'_>, shadows: &mut TagShadows) {
    if import.import_kind == ImportOrExportKind::Type {
        return;
    }
    let Some(specifiers) = &import.specifiers else {
        return;
    };
    for specifier in specifiers {
        let local = match specifier {
            ImportDeclarationSpecifier::ImportSpecifier(named) => {
                if named.import_kind == ImportOrExportKind::Type {
                    continue;
                }
                &named.local
            }
            ImportDeclarationSpecifier::ImportDefaultSpecifier(default) => &default.local,
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(namespace) => &namespace.local,
        };
        if local.name.eq_ignore_ascii_case("sql") {
            shadows.names.insert(local.name.to_string());
        }
    }
}

fn record_declaration(
    declaration: &VariableDeclaration<'_>,
    top_level_functions: &HashSet<&str>,
    shadows: &mut TagShadows,
) {
    for declarator in &declaration.declarations {
        record_declarator(declarator, top_level_functions, shadows);
    }
}

/// A destructured declarator (`const { tag: sql } = providers;`) has no
/// single callable-helper shape to exempt — the callable-helper exemption
/// below applies only to a simple `BindingIdentifier` — so every name it
/// binds is recorded as a shadow, matching how a non-function-shaped
/// identifier declarator is already handled. A name that also names a
/// top-level `function` declaration is exempted unconditionally, even from
/// the `sql`-specific override just below: see
/// [`top_level_function_names`].
fn record_declarator(
    declarator: &VariableDeclarator<'_>,
    top_level_functions: &HashSet<&str>,
    shadows: &mut TagShadows,
) {
    let is_helper_shape = declarator.init.as_ref().is_some_and(|init| {
        matches!(&declarator.id, BindingPattern::BindingIdentifier(_)) && is_function_shaped(init)
    });
    for_each_bound_name(&declarator.id, &mut |name| {
        if top_level_functions.contains(name) {
            return;
        }
        if !is_helper_shape || name.eq_ignore_ascii_case("sql") {
            shadows.names.insert(name.to_string());
        }
    });
}

pub(super) fn is_function_shaped(expr: &Expression<'_>) -> bool {
    matches!(
        unwrap_ts_wrappers(expr),
        Expression::FunctionExpression(_) | Expression::ArrowFunctionExpression(_)
    )
}

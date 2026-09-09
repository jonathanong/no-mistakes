mod import;

use super::super::for_each_bound_name;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{
    BindingPattern, Class, Declaration, Expression, Function, Program, Statement,
    VariableDeclaration, VariableDeclarator,
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
/// `sql` (case-insensitively) or `String`: those are the only names
/// [`crate::codebase::postgres::embedded::tags`] will treat as a trusted
/// tagged-template tag, and a callable rebinding of either is exactly the
/// shape that can ignore its template arguments and return arbitrary text.
///
/// Default imports from `sql-template-strings` are the opposite: they *are*
/// the trusted tag, recorded in [`TagShadows::imported`] under whatever
/// local name the file used.
#[derive(Default)]
pub(super) struct TagShadows {
    names: HashSet<String>,
    imported: HashSet<String>,
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

    pub(super) fn imported_tags(&self) -> &HashSet<String> {
        &self.imported
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
        Statement::FunctionDeclaration(function) => {
            record_function_tag_shadow(function, shadows);
        }
        Statement::ClassDeclaration(class) => {
            record_class_tag_shadow(class, shadows);
        }
        Statement::ExportDeclaration(export) => match &export.declaration {
            Declaration::VariableDeclaration(declaration) => {
                record_declaration(declaration, top_level_functions, shadows);
            }
            Declaration::FunctionDeclaration(function) => {
                record_function_tag_shadow(function, shadows);
            }
            Declaration::ClassDeclaration(class) => {
                record_class_tag_shadow(class, shadows);
            }
            _ => {}
        },
        Statement::ImportDeclaration(import) => import::record_import(import, shadows),
        _ => {}
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
        if !is_helper_shape || is_trusted_tag_name(name) {
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

pub(super) fn is_trusted_tag_name(name: &str) -> bool {
    name.eq_ignore_ascii_case("sql") || name == "String"
}

/// A `function sql()` / `function String()` whose body is a static SQL
/// fragment is a helper, not a tagged-template implementation — so the name
/// must not be trusted as the intrinsic tag. A tag implementation such as
/// `function sql(strings, ...values) { return strings.join(""); }` is left
/// alone so in-file tag fixtures keep working.
fn record_function_tag_shadow(function: &Function<'_>, shadows: &mut TagShadows) {
    let Some(id) = function.id.as_ref() else {
        return;
    };
    if !is_trusted_tag_name(id.name.as_str()) {
        return;
    }
    if returns_static_sql(function) {
        shadows.names.insert(id.name.to_string());
    }
}

fn record_class_tag_shadow(class: &Class<'_>, shadows: &mut TagShadows) {
    let Some(id) = class.id.as_ref() else {
        return;
    };
    if is_trusted_tag_name(id.name.as_str()) {
        shadows.names.insert(id.name.to_string());
    }
}

fn returns_static_sql(function: &Function<'_>) -> bool {
    let Some(body) = function.body.as_ref() else {
        return false;
    };
    let [oxc_ast::ast::Statement::ReturnStatement(ret)] = body.statements.as_slice() else {
        return false;
    };
    let Some(argument) = ret.argument.as_ref() else {
        return false;
    };
    matches!(
        unwrap_ts_wrappers(argument),
        Expression::StringLiteral(_)
            | Expression::TemplateLiteral(_)
            | Expression::TaggedTemplateExpression(_)
    )
}

use super::chain;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{
    ArrowFunctionBody, AssignmentExpression, AssignmentTarget, BindingPattern, Declaration,
    Expression, FormalParameters, Function, FunctionBody, Program, Statement, VariableDeclaration,
    VariableDeclarationKind, VariableDeclarator,
};
use oxc_ast_visit::{walk, Visit};
use std::collections::{HashMap, HashSet};

pub(super) const MAX_RESOLVE_DEPTH: u8 = 8;

/// Same-file functions whose body is a single `return` of a statically
/// resolvable SQL fragment, pre-resolved once per file.
///
/// Collection only walks `program.body` — a helper declared lexically nested
/// inside another function or block is never collected, so a call to it
/// fails closed (`Dynamic`) rather than risking resolving through the wrong
/// binding. This is an accepted limitation, not a soundness gap: extending
/// collection to nested scopes only widens what resolves as `Composed`, it
/// never narrows it.
pub(crate) struct LocalFunctions {
    resolved: HashMap<String, String>,
}

/// A same-file helper's params and body, however it was declared
/// (`function`, `const x = function() {}`, or `const x = () => {}`).
///
/// A single lifetime, not two: [`unwrap_ts_wrappers`] requires its argument's
/// reference and arena lifetimes to be the same, so any type built from its
/// result must use one lifetime throughout rather than distinguishing a
/// "place" lifetime from an "arena" lifetime.
struct Resolvable<'a> {
    params: &'a FormalParameters<'a>,
    body: &'a FunctionBody<'a>,
}

impl LocalFunctions {
    pub(crate) fn collect(program: &Program<'_>) -> Self {
        let mut raw: HashMap<&str, Resolvable<'_>> = HashMap::new();
        for statement in &program.body {
            collect_named_functions(statement, &mut raw);
        }
        let mut reassigned = ReassignedNames::default();
        reassigned.visit_program(program);
        raw.retain(|name, _| !reassigned.names.contains(name));
        let mut resolved = HashMap::new();
        for name in raw.keys().copied() {
            let mut resolving = Vec::new();
            if let Some(text) = resolve_named(name, MAX_RESOLVE_DEPTH, &raw, &mut resolving) {
                resolved.insert(name.to_string(), text);
            }
        }
        Self { resolved }
    }

    pub(crate) fn get(&self, name: &str) -> Option<String> {
        self.resolved.get(name).cloned()
    }
}

/// Names assigned anywhere in the program, e.g. `build = externalBuilder;`
/// reassigning a hoisted `function build() {}`. A function declaration's
/// binding is mutable, so a call to it can no longer be trusted to run the
/// originally-collected body once any assignment to that name exists
/// anywhere — `LocalFunctions::collect` drops such names outright rather
/// than resolving through a body that may not be the one that runs.
#[derive(Default)]
struct ReassignedNames<'a> {
    names: HashSet<&'a str>,
}

impl<'a> Visit<'a> for ReassignedNames<'a> {
    fn visit_assignment_expression(&mut self, assign: &AssignmentExpression<'a>) {
        if let AssignmentTarget::AssignmentTargetIdentifier(ident) = &assign.left {
            self.names.insert(ident.name.as_str());
        }
        walk::walk_assignment_expression(self, assign);
    }
}

fn collect_named_functions<'a>(
    statement: &'a Statement<'a>,
    raw: &mut HashMap<&'a str, Resolvable<'a>>,
) {
    match statement {
        Statement::FunctionDeclaration(function) => insert_function(function, raw),
        Statement::VariableDeclaration(declaration) => insert_const_functions(declaration, raw),
        Statement::ExportDeclaration(export) => match &export.declaration {
            Declaration::FunctionDeclaration(function) => insert_function(function, raw),
            Declaration::VariableDeclaration(declaration) => {
                insert_const_functions(declaration, raw);
            }
            _ => {}
        },
        _ => {}
    }
}

fn insert_function<'a>(function: &'a Function<'a>, raw: &mut HashMap<&'a str, Resolvable<'a>>) {
    if let Some((name, resolvable)) = function_resolvable(function) {
        raw.insert(name, resolvable);
    }
}

fn insert_const_functions<'a>(
    declaration: &'a VariableDeclaration<'a>,
    raw: &mut HashMap<&'a str, Resolvable<'a>>,
) {
    if declaration.kind != VariableDeclarationKind::Const {
        return;
    }
    for declarator in &declaration.declarations {
        if let Some((name, resolvable)) = const_resolvable(declarator) {
            raw.insert(name, resolvable);
        }
    }
}

fn function_resolvable<'a>(function: &'a Function<'a>) -> Option<(&'a str, Resolvable<'a>)> {
    let id = function.id.as_ref()?;
    let resolvable = resolvable_body(function)?;
    Some((id.name.as_str(), resolvable))
}

/// A function's params and body, provided it's synchronously inlinable —
/// irrespective of whether it has a name of its own. A `const`-bound
/// function expression (`const x = function () {...}`) is anonymous at the
/// AST level; its name comes from the binding, not [`function_resolvable`]'s
/// `function.id`.
fn resolvable_body<'a>(function: &'a Function<'a>) -> Option<Resolvable<'a>> {
    if function.r#async || function.generator {
        return None;
    }
    let body = function.body.as_ref()?;
    Some(Resolvable {
        params: &function.params,
        body,
    })
}

fn const_resolvable<'a>(
    declarator: &'a VariableDeclarator<'a>,
) -> Option<(&'a str, Resolvable<'a>)> {
    let BindingPattern::BindingIdentifier(ident) = &declarator.id else {
        return None;
    };
    let init = declarator.init.as_ref()?;
    match unwrap_ts_wrappers(init) {
        Expression::FunctionExpression(function) => {
            let resolvable = resolvable_body(function)?;
            Some((ident.name.as_str(), resolvable))
        }
        Expression::ArrowFunctionExpression(arrow) => {
            if arrow.r#async {
                return None;
            }
            let ArrowFunctionBody::FunctionBody(body) = &arrow.body else {
                return None;
            };
            Some((
                ident.name.as_str(),
                Resolvable {
                    params: &arrow.params,
                    body,
                },
            ))
        }
        _ => None,
    }
}

fn shadows_param(resolvable: &Resolvable<'_>, name: &str) -> bool {
    resolvable.params.items.iter().any(|param| {
        let mut shadows = false;
        super::for_each_bound_name(&param.pattern, &mut |bound| shadows |= bound == name);
        shadows
    })
}

/// A function only inlines when its body is exactly one `return <expr>;` —
/// no local declarations, no control flow, no side effects to reason about.
/// Parameters used outside a template placeholder never resolve, because
/// `chain::resolve_expr` has no `Identifier` case: that keeps this sound
/// without a separate parameter-position check. A callee that shadows one of
/// this function's own parameters is rejected rather than resolved through
/// the global declaration of the same name.
fn resolve_named(
    name: &str,
    depth: u8,
    raw: &HashMap<&str, Resolvable<'_>>,
    resolving: &mut Vec<String>,
) -> Option<String> {
    if resolving.iter().any(|seen| seen == name) {
        return None;
    }
    let resolvable = raw.get(name)?;
    let [Statement::ReturnStatement(ret)] = resolvable.body.statements.as_slice() else {
        return None;
    };
    let argument = ret.argument.as_ref()?;
    resolving.push(name.to_string());
    let mut lookup = |callee: &str, depth: u8| {
        if shadows_param(resolvable, callee) {
            return None;
        }
        resolve_named(callee, depth, raw, resolving)
    };
    let text = chain::resolve_expr(argument, depth, &mut lookup);
    resolving.pop();
    text
}

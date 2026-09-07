use super::chain;
use oxc_ast::ast::{Declaration, Function, Program, Statement};
use std::collections::HashMap;

pub(super) const MAX_RESOLVE_DEPTH: u8 = 8;

/// Same-file functions whose body is a single `return` of a statically
/// resolvable SQL fragment, pre-resolved once per file.
pub(crate) struct LocalFunctions {
    resolved: HashMap<String, String>,
}

impl LocalFunctions {
    pub(crate) fn collect(program: &Program<'_>) -> Self {
        let mut raw: HashMap<&str, &Function<'_>> = HashMap::new();
        for statement in &program.body {
            if let Some((name, function)) = named_function(statement) {
                raw.insert(name, function);
            }
        }
        let mut resolved = HashMap::new();
        for name in raw.keys().copied() {
            let mut resolving = Vec::new();
            if let Some(text) = resolve_named(name, &raw, &mut resolving) {
                resolved.insert(name.to_string(), text);
            }
        }
        Self { resolved }
    }

    pub(crate) fn get(&self, name: &str) -> Option<String> {
        self.resolved.get(name).cloned()
    }
}

fn named_function<'p, 'a>(statement: &'p Statement<'a>) -> Option<(&'p str, &'p Function<'a>)> {
    let function = match statement {
        Statement::FunctionDeclaration(function) => function.as_ref(),
        Statement::ExportDeclaration(export) => match &export.declaration {
            Declaration::FunctionDeclaration(function) => function.as_ref(),
            _ => return None,
        },
        _ => return None,
    };
    let id = function.id.as_ref()?;
    Some((id.name.as_str(), function))
}

/// A function only inlines when its body is exactly one `return <expr>;` —
/// no local declarations, no control flow, no side effects to reason about.
/// Parameters used outside a template placeholder never resolve, because
/// `chain::resolve_expr` has no `Identifier` case: that keeps this sound
/// without a separate parameter-position check.
fn resolve_named(
    name: &str,
    raw: &HashMap<&str, &Function<'_>>,
    resolving: &mut Vec<String>,
) -> Option<String> {
    if resolving.iter().any(|seen| seen == name) {
        return None;
    }
    let function = *raw.get(name)?;
    let body = function.body.as_ref()?;
    let [Statement::ReturnStatement(ret)] = body.statements.as_slice() else {
        return None;
    };
    let argument = ret.argument.as_ref()?;
    resolving.push(name.to_string());
    let mut lookup = |callee: &str, _depth: u8| resolve_named(callee, raw, resolving);
    let text = chain::resolve_expr(argument, MAX_RESOLVE_DEPTH, &mut lookup);
    resolving.pop();
    text
}

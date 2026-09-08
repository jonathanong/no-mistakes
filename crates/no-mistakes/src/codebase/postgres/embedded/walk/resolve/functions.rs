mod collect;
mod reassigned;
mod shadows;

use super::chain;
use collect::collect_named_functions;
use oxc_ast::ast::{FormalParameters, FunctionBody, Program, Statement};
use reassigned::ReassignedNames;
use shadows::TagShadows;
use std::collections::HashMap;

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
        let reassigned = ReassignedNames::collect(program);
        raw.retain(|name, _| !reassigned.contains(name));
        let tag_shadows = TagShadows::collect(program);
        let mut resolved = HashMap::new();
        for name in raw.keys().copied() {
            let mut resolving = Vec::new();
            if let Some(text) =
                resolve_named(name, MAX_RESOLVE_DEPTH, &raw, &mut resolving, &tag_shadows)
            {
                resolved.insert(name.to_string(), text);
            }
        }
        Self { resolved }
    }

    pub(crate) fn get(&self, name: &str) -> Option<String> {
        self.resolved.get(name).cloned()
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
/// the global declaration of the same name — and so is a tagged template
/// whose tag name (e.g. `sql`) is one of this function's own parameters, or
/// is rebound anywhere else at the top level by something other than a
/// same-file helper (see [`shadows::TagShadows`]), via `is_shadowed`.
fn resolve_named(
    name: &str,
    depth: u8,
    raw: &HashMap<&str, Resolvable<'_>>,
    resolving: &mut Vec<String>,
    tag_shadows: &TagShadows,
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
        resolve_named(callee, depth, raw, resolving, tag_shadows)
    };
    let mut is_shadowed = |tag: &str| shadows_param(resolvable, tag) || tag_shadows.contains(tag);
    let text = chain::resolve_expr(argument, depth, &mut lookup, &mut is_shadowed);
    resolving.pop();
    text
}

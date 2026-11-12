//! Bounded invocation-local literal facts; never evaluate source code.
use crate::fx::{FxHashMap, FxHashSet};
use oxc_ast::{ast::*, AstKind};
use oxc_semantic::Semantic;
use serde_json::Value;
mod aliases;
use aliases::AliasSafety;
mod path_origin;
mod path_origin_equality;
mod path_value;
mod provenance;
#[path = "arguments/provenance/pass_through_writes.rs"]
mod transparent_writes;
pub(crate) use transparent_writes::{
    transparent_receiver_writes, transparent_receiver_writes_from_program,
};
mod captured_env;
pub(crate) use captured_env::CapturedEnv;
mod env_diagnostic;
pub(crate) use env_diagnostic::EnvDiagnostic;
mod types;
mod value;
pub(crate) use types::{
    line_in, ArgumentText, ArgumentValue, ArgumentValues, ArrayGuard, EnvWrite, MemberEffect,
    PathAnchor, PathDefault, PathOrigin, PathProvenance, PathValueKind, RouteKind, RouteSite,
    WriteScope,
};
pub(super) fn collect(
    semantic: &Semantic<'_>,
    facts: &super::TypeScriptModuleFacts,
) -> (FxHashMap<u32, ArgumentValues>, PathProvenance) {
    let names: FxHashSet<_> = facts
        .exports
        .iter()
        .filter(|e| e.specifier.is_none() && !e.type_only)
        .map(|e| e.local.as_str())
        .collect();
    let roots: FxHashSet<_> = facts
        .scopes
        .iter()
        .filter(|s| s.parent_id.is_none())
        .map(|s| s.id)
        .collect();
    let exported: FxHashSet<_> = facts
        .bindings
        .iter()
        .filter(|b| roots.contains(&b.scope_id) && names.contains(b.name.as_str()))
        .map(|b| b.id)
        .collect();
    let temp_factories = path_origin::temp_factories(semantic, facts);
    let imported = path_origin::imported_locals(facts);
    let mut provenance = provenance::collect(semantic, facts, &exported, &temp_factories);
    let mut immutable = AliasSafety::default();
    let mut result = FxHashMap::default();
    let mut dynamic_scope = false;
    for node in semantic.nodes().iter() {
        if matches!(node.kind(), AstKind::WithStatement(_)) {
            dynamic_scope = true;
        }
        if let AstKind::CallExpression(call) = node.kind() {
            if let Expression::Identifier(id) = call.callee.get_inner_expression() {
                if id.name == "eval"
                    && id
                        .reference_id
                        .get()
                        .is_some_and(|r| semantic.scoping().get_reference(r).symbol_id().is_none())
                {
                    dynamic_scope = true;
                }
            }
        }
        let (offset, arguments, callee) = match node.kind() {
            AstKind::CallExpression(call) => (call.span.start, &call.arguments, Some(call)),
            AstKind::NewExpression(call) => (call.span.start, &call.arguments, None),
            _ => continue,
        };
        let mut projector = Projector {
            semantic,
            call_offset: offset,
            remaining: 256,
            exported: &exported,
            immutable: &mut immutable,
            temp_factories: &temp_factories,
        };
        let mut spread = false;
        let spread_from = arguments
            .iter()
            .position(|argument| argument.as_expression().is_none());
        let values = arguments
            .iter()
            .take(32)
            .map(|argument| {
                let Some(expression) = argument.as_expression() else {
                    spread = true;
                    return ArgumentValue::Unknown;
                };
                if spread {
                    ArgumentValue::Unknown
                } else {
                    projector.value(expression, 0)
                }
            })
            .collect();
        let path = callee.and_then(|call| projector.call_path(call, &imported));
        result.insert(
            offset,
            ArgumentValues {
                values,
                spread_from,
                path,
            },
        );
    }
    if dynamic_scope {
        provenance.call_guards.clear();
        for call in result.values_mut() {
            call.values.fill(ArgumentValue::Unknown);
            call.spread_from = Some(0);
            call.path = call.path.take().map(|_| PathOrigin::unknown());
        }
    }
    (result, provenance)
}
struct Projector<'a, 's> {
    semantic: &'s Semantic<'a>,
    call_offset: u32,
    remaining: usize,
    exported: &'s FxHashSet<usize>,
    immutable: &'s mut AliasSafety,
    temp_factories: &'s path_origin::TempFactories,
}

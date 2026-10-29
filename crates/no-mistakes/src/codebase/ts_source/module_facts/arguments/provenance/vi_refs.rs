use super::super::PathProvenance;
use crate::fx::FxHashSet;
use oxc_ast::{ast::*, AstKind};
use oxc_semantic::AstNodes;
use oxc_span::GetSpan;
use oxc_syntax::node::NodeId;

pub(super) const RUNNER_MODULES: [&str; 2] = ["vitest", "@jest/globals"];

const HELPERS: [&str; 34] = [
    "fn",
    "spyOn",
    "mock",
    "doMock",
    "unmock",
    "doUnmock",
    "mocked",
    "hoisted",
    "importActual",
    "importMock",
    "requireActual",
    "requireMock",
    "isMockFunction",
    "clearAllMocks",
    "resetAllMocks",
    "restoreAllMocks",
    "resetModules",
    "useFakeTimers",
    "useRealTimers",
    "advanceTimersByTime",
    "advanceTimersByTimeAsync",
    "advanceTimersToNextTimer",
    "runAllTimers",
    "runAllTimersAsync",
    "runOnlyPendingTimers",
    "runOnlyPendingTimersAsync",
    "setSystemTime",
    "getMockedSystemTime",
    "getRealSystemTime",
    "clearAllTimers",
    "waitFor",
    "waitUntil",
    "isFakeTimers",
    "stubEnv",
];

pub(super) fn local_names(
    facts: &super::super::super::TypeScriptModuleFacts,
    out: &mut PathProvenance,
) -> FxHashSet<String> {
    let mut names = FxHashSet::default();
    for import in &facts.imports {
        let runner = RUNNER_MODULES.contains(&import.specifier.as_str());
        for binding in &import.bindings {
            if runner && binding.kind != "named" {
                out.env_unknown_write = true;
            }
            if matches!(binding.imported.as_str(), "vi" | "jest") {
                names.insert(binding.local.clone());
            }
        }
    }
    names
}

pub(super) fn check(
    nodes: &AstNodes<'_>,
    node: NodeId,
    id: &IdentifierReference<'_>,
    names: &FxHashSet<String>,
    out: &mut PathProvenance,
) {
    if !(matches!(id.name.as_str(), "vi" | "jest") || names.contains(id.name.as_str())) {
        return;
    }
    let safe = match nodes.parent_kind(node) {
        AstKind::StaticMemberExpression(member) if member.object.span() == id.span => {
            HELPERS.contains(&member.property.name.as_str())
                && matches!(nodes.parent_kind(nodes.parent_id(node)),
                    AstKind::CallExpression(call) if call.callee.span() == member.span
                        && (member.property.name != "spyOn" || spies_on_a_plain_target(call)))
        }
        _ => false,
    };
    if !safe {
        out.env_unknown_write = true;
    }
}

/// `spyOn` reaches `process` only through the global object or a `process` argument,
/// both poisoned where they appear; any other computed target is not understood.
fn spies_on_a_plain_target(call: &CallExpression<'_>) -> bool {
    matches!(
        call.arguments
            .first()
            .and_then(Argument::as_expression)
            .map(Expression::get_inner_expression),
        Some(Expression::Identifier(_) | Expression::StaticMemberExpression(_))
    )
}

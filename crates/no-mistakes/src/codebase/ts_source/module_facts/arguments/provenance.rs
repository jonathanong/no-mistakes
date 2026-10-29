//! Call-argument origins, spread-forwarding mocks, and `process.env` writes.
//!
//! Syntactic only: a form this pass does not recognise is left out, and the
//! rule that reads these facts then keeps its finding.
mod consumers;
mod env_writes;
mod pass_through;
mod process_refs;
mod scope;
mod value_refs;
mod vi_refs;

use super::path_origin::{callee_root, imported_locals};
use super::{PathAnchor, PathOrigin, PathProvenance, Projector};
use crate::fx::FxHashSet;
use oxc_ast::{ast::*, AstKind};
use oxc_semantic::Semantic;

/// Arguments after this index are never recorded.
const MAX_ARGUMENTS: usize = 8;

pub(super) fn collect(
    semantic: &Semantic<'_>,
    facts: &super::super::TypeScriptModuleFacts,
    exported: &FxHashSet<usize>,
) -> PathProvenance {
    let imported = imported_locals(facts);
    let mut immutable = super::AliasSafety::default();
    let mut result = PathProvenance::default();
    process_refs::poison_process_imports(facts, &mut result);
    let vi_names = vi_refs::local_names(facts, &mut result);
    for node in semantic.nodes().iter() {
        let mut projector = Projector {
            semantic,
            call_offset: 0,
            remaining: 256,
            exported,
            immutable: &mut immutable,
        };
        match node.kind() {
            AstKind::CallExpression(call) => {
                projector.call_offset = call.span.start;
                if let Some(spied) =
                    pass_through::forwards_mock_arguments(semantic, node.id(), call)
                {
                    result.pass_through.insert(call.span.start, spied);
                }
                consumers::from_call(call, &mut result);
                if callee_root(&call.callee).is_some_and(|root| imported.contains(root))
                    && !rebinds_callee(&call.callee)
                {
                    let paths = argument_paths(&mut projector, call);
                    if !paths.is_empty() {
                        result.arg_paths.insert(call.span.start, paths);
                    }
                }
                if let Some(span) = test_callback(call) {
                    result.test_callbacks.push(span);
                }
                env_writes::from_call(&mut projector, call, node.id(), &mut result);
            }
            AstKind::IdentifierReference(id) => {
                process_refs::from_identifier(semantic, node.id(), id, &mut result);
                process_refs::poison_name(&id.name, &mut result);
                vi_refs::check(semantic.nodes(), node.id(), id, &vi_names, &mut result);
                value_refs::from_identifier(semantic, node.id(), id, &mut result);
            }
            AstKind::IdentifierName(name) => process_refs::poison_name(&name.name, &mut result),
            AstKind::StringLiteral(text) => {
                process_refs::poison_name(&crate::ast::js_literal(text), &mut result);
            }
            AstKind::StaticMemberExpression(member) => {
                value_refs::from_member(semantic.nodes(), node.id(), member, &mut result);
            }
            AstKind::ImportExpression(import) => {
                consumers::from_source(Some(&import.source), &mut result);
            }
            AstKind::AssignmentExpression(assignment) => {
                env_writes::from_assignment(&mut projector, assignment, node.id(), &mut result);
            }
            AstKind::UnaryExpression(unary) => {
                env_writes::from_delete(&mut projector, unary, node.id(), &mut result);
            }
            AstKind::VariableDeclarator(declarator) => {
                env_writes::from_alias(&mut projector, declarator, &mut result);
            }
            _ => {}
        }
    }
    value_refs::add_imported_names(facts, &mut result);
    consumers::from_facts(facts, &mut result);
    result
}

/// `sync.call(...)`, `.apply`, and `.bind` shift argument positions.
fn rebinds_callee(callee: &Expression<'_>) -> bool {
    matches!(callee.get_inner_expression(), Expression::StaticMemberExpression(member)
        if matches!(member.property.name.as_str(), "call" | "apply" | "bind"))
}

/// Only anchors that can prove an owned or fixture root are kept, which bounds memory.
fn argument_paths<'a>(
    projector: &mut Projector<'a, '_>,
    call: &CallExpression<'a>,
) -> Vec<(usize, PathOrigin)> {
    let mut paths = Vec::new();
    for (index, argument) in call.arguments.iter().take(MAX_ARGUMENTS).enumerate() {
        let Some(expression) = argument.as_expression() else {
            break;
        };
        let origin = projector.path_of(expression);
        if matches!(
            origin.anchor,
            PathAnchor::Temp | PathAnchor::Module | PathAnchor::Cwd
        ) {
            paths.push((index, origin));
        }
    }
    paths
}

/// `it(...)` / `test(...)` (including `.only` / `.skip`) with a function body.
fn test_callback(call: &CallExpression<'_>) -> Option<(u32, u32)> {
    if !matches!(callee_root(&call.callee)?, "it" | "test") {
        return None;
    }
    call.arguments.iter().find_map(|argument| {
        match argument.as_expression()?.get_inner_expression() {
            Expression::ArrowFunctionExpression(function) => {
                Some((function.span.start, function.span.end))
            }
            Expression::FunctionExpression(function) => {
                Some((function.span.start, function.span.end))
            }
            _ => None,
        }
    })
}

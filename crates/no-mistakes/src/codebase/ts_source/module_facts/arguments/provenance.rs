//! Call-argument origins, spread-forwarding mocks, and `process.env` writes.
//!
//! Syntactic only: a form this pass does not recognise is left out, and the
//! rule that reads these facts then keeps its finding.
mod argument_source;
mod call_args;
mod concurrent;
mod config_keys;
mod consumers;
mod env_writes;
mod getters;
mod local_env;
mod member_paths;
mod member_refs;
pub(super) mod pass_through;
mod process_refs;
mod resets;
mod scope;
mod stub_global;
mod value_refs;
mod vi_refs;

use super::path_origin::callee_root;
use super::{PathProvenance, Projector};
use crate::fx::FxHashSet;
use oxc_ast::AstKind;
use oxc_semantic::Semantic;

pub(super) fn collect(
    semantic: &Semantic<'_>,
    facts: &super::super::TypeScriptModuleFacts,
    exported: &FxHashSet<usize>,
    temp_factories: &super::path_origin::TempFactories,
) -> PathProvenance {
    let mut immutable = super::AliasSafety::default();
    let mut result = PathProvenance::default();
    process_refs::poison_process_imports(facts, &mut result);
    let vi_names = vi_refs::local_names(facts, &mut result);
    let source = argument_source::ArgumentSource::new(semantic.source_text());
    for node in semantic.nodes().iter() {
        let mut projector = Projector {
            semantic,
            call_offset: 0,
            remaining: 256,
            exported,
            immutable: &mut immutable,
            temp_factories,
        };
        match node.kind() {
            AstKind::Function(function) => getters::record(&mut projector, function, &mut result),
            AstKind::ArrowFunctionExpression(function) => getters::record_defaults(
                &mut projector,
                function.span.start,
                &function.params,
                &mut result,
            ),
            AstKind::CallExpression(call) => {
                if concurrent::overlap(semantic, call) {
                    result
                        .lifecycle_overlaps
                        .extend(concurrent::callbacks(semantic, call));
                }
                projector.call_offset = call.span.start;
                if let Some(spied) =
                    pass_through::forwards_mock_arguments(semantic, node.id(), call)
                {
                    result.pass_through.insert(call.span.start, spied);
                }
                consumers::from_call(&mut projector, call, &mut result);
                if callee_root(&call.callee).is_some() && !call_args::rebinds_callee(&call.callee) {
                    let (members, texts) =
                        member_paths::collect(&mut projector, call, &source, &mut result);
                    if !members.is_empty() {
                        result.arg_member_paths.insert(call.span.start, members);
                    }
                    call_args::record_arg_text(&source, call, &mut result, texts);
                    let paths = call_args::argument_paths(&mut projector, call, &mut result);
                    if !paths.is_empty() {
                        if paths
                            .iter()
                            .any(|(_, path)| path.atoms().iter().any(|atom| atom.env.is_some()))
                            && local_env::immediate_read(semantic, node.id())
                        {
                            result.immediate_env_reads.insert(call.span.start);
                        }
                        result.arg_paths.insert(call.span.start, paths);
                    }
                }
                if let Some(span) = call_args::test_callback(call) {
                    result.test_callbacks.push(span);
                    local_env::test_calls(semantic, facts, call, &mut result);
                }
                if let Some(span) = resets::callback(semantic, call) {
                    result.cleanup_callbacks.push(span);
                }
                env_writes::from_call(&mut projector, call, node.id(), &mut result);
            }
            AstKind::IdentifierReference(id) => {
                process_refs::from_identifier(semantic, node.id(), id, &mut result);
                process_refs::poison_name(&id.name, semantic, facts, node.id(), &mut result);
                vi_refs::check(semantic.nodes(), node.id(), id, &vi_names, &mut result);
                value_refs::from_identifier(semantic, node.id(), id, &mut result);
            }
            AstKind::IdentifierName(name) => {
                process_refs::poison_name(&name.name, semantic, facts, node.id(), &mut result);
                member_refs::from_name(semantic.nodes(), node.id(), &name.name, &mut result);
            }
            AstKind::StringLiteral(text) => {
                let literal = crate::ast::js_literal(text);
                process_refs::poison_name(&literal, semantic, facts, node.id(), &mut result);
                member_refs::from_literal(&literal, &mut result);
            }
            AstKind::ComputedMemberExpression(member) => {
                member_refs::from_computed(member, &mut result);
            }
            AstKind::ImportExpression(import) => {
                projector.call_offset = import.span.start;
                if consumers::from_source(
                    &mut projector,
                    Some(&import.source),
                    &mut result,
                    import.span.start,
                    crate::codebase::ts_source::module_facts::arguments::RouteKind::Dynamic,
                ) {
                    result
                        .resolved_import_spans
                        .push((import.span.start, import.span.end));
                }
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
            AstKind::ObjectProperty(property) => {
                config_keys::from_property(property, &mut result);
            }
            _ => {}
        }
    }
    value_refs::add_imported_names(facts, &mut result);
    consumers::from_facts(facts, semantic.source_text(), &mut result);
    result
}

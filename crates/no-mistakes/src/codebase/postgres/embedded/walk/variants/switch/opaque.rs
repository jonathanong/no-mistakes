use super::super::super::{BindingState, ScopeVisitor};
use oxc_ast::ast::SwitchStatement;
use oxc_ast_visit::Visit;
use std::collections::{hash_map::Entry, HashMap};

pub(super) fn visit_cases<'a>(visitor: &mut ScopeVisitor<'a>, statement: &SwitchStatement<'a>) {
    let mut entry = visitor.scopes.clone();
    visitor.with_loop(|visitor| {
        for case in &statement.cases {
            // Each body can be entered directly without running an earlier
            // body. Its SQL cannot come solely from an earlier body's writes.
            visitor.scopes = entry.clone();
            if let Some(test) = &case.test {
                visitor.visit_expression(test);
            }
            visitor.with_control_flow(|visitor| {
                for statement in &case.consequent {
                    visitor.visit_statement(statement);
                }
            });
            fold_changes(&mut entry, &visitor.scopes);
        }
    });
    visitor.scopes = entry;
}

fn fold_changes(
    entry: &mut [HashMap<String, BindingState>],
    current: &[HashMap<String, BindingState>],
) {
    for (entry, current) in entry.iter_mut().zip(current) {
        for (name, state) in current {
            match entry.entry(name.clone()) {
                Entry::Occupied(mut binding) => {
                    if changed(binding.get(), state) {
                        let binding = binding.get_mut();
                        binding.initialized |= state.initialized;
                        binding.sql_builder |= state.sql_builder;
                        binding.builder_effect_revision = state.builder_effect_revision;
                        make_opaque(binding);
                    }
                }
                // An escaped captured builder may gain a function-local
                // snapshot while a body is visited. Retain only its uncertainty.
                Entry::Vacant(binding) => {
                    let mut state = state.clone();
                    make_opaque(&mut state);
                    binding.insert(state);
                }
            }
        }
    }
}

fn changed(before: &BindingState, after: &BindingState) -> bool {
    (
        &before.sql,
        before.kind,
        before.builder_identity,
        &before.variants,
        before.builder_effect_revision,
        before.condition_key,
        before.initialized,
        before.sql_builder,
        &before.sql_source_positions,
    ) != (
        &after.sql,
        after.kind,
        after.builder_identity,
        &after.variants,
        after.builder_effect_revision,
        after.condition_key,
        after.initialized,
        after.sql_builder,
        &after.sql_source_positions,
    )
}

fn make_opaque(binding: &mut BindingState) {
    // Direct entry and fallthrough may see different statements, even different
    // leading DML kinds. Neither path supplies one shared, verified SQL prefix.
    binding.kind = super::super::super::super::EmbeddedSqlKind::Dynamic;
    binding.sql = None;
    binding.sql_source_positions.clear();
    binding.variants = None;
    binding.condition_key = None;
    binding.builder_identity = None;
}

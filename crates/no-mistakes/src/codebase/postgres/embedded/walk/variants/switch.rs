use super::super::{resolve, ScopeVisitor};
use super::control::{branch_paths, Snapshot};
use oxc_ast::ast::{Statement, SwitchStatement};
use oxc_ast_visit::Visit;
mod state;
use state::join;

pub(in crate::codebase::postgres::embedded::walk) fn statement<'a>(
    visitor: &mut ScopeVisitor<'a>,
    statement: &SwitchStatement<'a>,
) {
    let effectful = visitor.expression_mutates_builder(&statement.discriminant)
        || statement.cases.iter().any(|case| {
            case.test
                .as_ref()
                .is_some_and(|test| visitor.expression_mutates_builder(test))
        });
    visitor.visit_expression(&statement.discriminant);
    visitor.push_scope();
    for case in &statement.cases {
        resolve::record_statements(&case.consequent, visitor);
    }
    if effectful {
        // Failed labels execute cumulatively before the selected label. Their
        // mutations cannot use independent branch-entry SQL snapshots.
        visitor.forget_effectful_sql_prefixes(&statement.discriminant);
        for case in &statement.cases {
            if let Some(test) = &case.test {
                visitor.forget_effectful_sql_prefixes(test);
            }
        }
        visitor.invalidate_execution_variants();
        visitor.with_loop(|visitor| {
            for case in &statement.cases {
                if let Some(test) = &case.test {
                    visitor.visit_expression(test);
                }
                visitor.with_control_flow(|visitor| {
                    for statement in &case.consequent {
                        visitor.visit_statement(statement);
                    }
                });
            }
        });
        visitor.pop_scope();
        visitor.invalidate_execution_variants();
        return;
    }
    let entry = visitor.variant_snapshot();
    let paths = visitor.variant_paths.clone();
    let mut fallthrough: Option<Snapshot> = None;
    let mut fallthrough_paths = Vec::new();
    let mut exits: Option<Snapshot> = None;
    let mut exits_paths = Vec::new();
    for (index, case) in statement.cases.iter().enumerate() {
        visitor.restore_variants(&entry);
        let direct = branch_paths(&paths, u64::from(statement.span.start), index as u32);
        visitor.variant_paths = direct.clone();
        // A failed earlier label reaches this test from the entry state;
        // falling through an earlier body skips the test entirely.
        if let Some(test) = &case.test {
            visitor.visit_expression(test);
        }
        let direct_state = visitor.variant_snapshot();
        if let Some(prior) = &fallthrough {
            join(
                visitor,
                Some(prior),
                &fallthrough_paths,
                &direct_state,
                &direct,
                u64::from(statement.span.start),
            );
        }
        visitor.variant_paths.extend(fallthrough_paths.clone());
        let terminal_break = matches!(case.consequent.last(), Some(Statement::BreakStatement(break_)) if break_.label.is_none());
        let length = case.consequent.len() - usize::from(terminal_break);
        visitor.with_control_flow(|visitor| {
            for statement in &case.consequent[..length] {
                visitor.visit_statement(statement);
            }
        });
        let current = visitor.variant_snapshot();
        let current_paths = visitor.variant_paths.clone();
        if terminal_break {
            exits = Some(join(
                visitor,
                exits.as_ref(),
                &exits_paths,
                &current,
                &current_paths,
                u64::from(statement.span.start),
            ));
            exits_paths.extend(current_paths);
            fallthrough = None;
            fallthrough_paths.clear();
        } else {
            fallthrough = Some(current);
            fallthrough_paths = visitor.variant_paths.clone();
        }
    }
    if let Some(prior) = &fallthrough {
        exits = Some(join(
            visitor,
            exits.as_ref(),
            &exits_paths,
            prior,
            &fallthrough_paths,
            u64::from(statement.span.start),
        ));
        exits_paths.extend(fallthrough_paths);
    }
    if !statement.cases.iter().any(|case| case.test.is_none()) {
        visitor.restore_variants(&entry);
        visitor.variant_paths = branch_paths(
            &paths,
            u64::from(statement.span.start),
            statement.cases.len() as u32,
        );
        let absent = visitor.variant_snapshot();
        let absent_paths = visitor.variant_paths.clone();
        exits = Some(join(
            visitor,
            exits.as_ref(),
            &exits_paths,
            &absent,
            &absent_paths,
            u64::from(statement.span.start),
        ));
    }
    visitor.restore_variants(exits.as_ref().unwrap_or(&entry));
    state::select(visitor, statement);
    visitor.variant_paths = paths;
    visitor.pop_scope();
}

use super::super::super::transaction_flow::Snapshot as TransactionSnapshot;
use super::super::conditions;
use super::{abrupt, paths, select, truthiness, ScopeVisitor};
use oxc_ast::ast::IfStatement;
use oxc_ast_visit::Visit;

pub(in crate::codebase::postgres::embedded::walk) fn if_statement<'a>(
    visitor: &mut ScopeVisitor<'a>,
    statement: &IfStatement<'a>,
) {
    if visitor.executor_options.track_transaction_helpers {
        visit_if::<true>(visitor, statement);
    } else {
        visit_if::<false>(visitor, statement);
    }
}

fn visit_if<'a, const TRACK: bool>(visitor: &mut ScopeVisitor<'a>, statement: &IfStatement<'a>) {
    let (id, truth_arm) = conditions::choice_id(visitor, &statement.test, statement.span.start);
    let taken = conditions::truth_at(visitor, &statement.test);
    visitor.visit_variant_guard(&statement.test);
    let original = visitor.variant_snapshot();
    let paths = visitor.variant_paths.clone();
    let (yes, no) = truthiness::branches(visitor, &statement.test, &paths, id, truth_arm);
    let taken = paths::taken(&yes, &no).or(taken);
    let transaction_before = TRACK.then(|| TransactionSnapshot::capture(visitor));
    visitor.restore_variants(&paths::restrict(&original, &yes));
    visitor.variant_paths = yes;
    visit_arm::<TRACK>(visitor, &statement.consequent);
    let left = visitor.variant_snapshot();
    let transaction_left = transaction_before.as_ref().map(|before| {
        let left = TransactionSnapshot::capture(visitor);
        before.restore(visitor);
        left
    });
    visitor.restore_variants(&paths::restrict(&original, &no));
    visitor.variant_paths = no;
    if let Some(alternate) = &statement.alternate {
        visit_arm::<TRACK>(visitor, alternate);
    }
    let right = visitor.variant_snapshot();
    if let (Some(transaction_before), Some(transaction_left)) =
        (transaction_before, transaction_left)
    {
        let transaction_right = TransactionSnapshot::capture(visitor);
        let exits = super::super::super::transaction_flow::exits;
        match taken {
            Some(true) => transaction_left.restore(visitor),
            Some(false) => transaction_right.restore(visitor),
            None if exits(&statement.consequent) => {
                transaction_before.resume_after(&transaction_left, &transaction_right, visitor)
            }
            None if statement.alternate.as_ref().is_some_and(exits) => {
                transaction_before.resume_after(&transaction_right, &transaction_left, visitor)
            }
            None => transaction_left.join(&transaction_right, visitor),
        }
    }
    visitor.variant_paths = paths;
    match taken {
        Some(true) => select(visitor, &left, &right),
        Some(false) => select(visitor, &right, &left),
        None if abrupt::throws(&statement.consequent) => select(visitor, &right, &left),
        None if statement
            .alternate
            .as_ref()
            .is_some_and(|arm| abrupt::throws(arm)) =>
        {
            select(visitor, &left, &right)
        }
        None => visitor.join_variants(&left, &right, id, (truth_arm, 1 - truth_arm)),
    }
}

fn visit_arm<'a, const TRACK: bool>(
    visitor: &mut ScopeVisitor<'a>,
    statement: &oxc_ast::ast::Statement<'a>,
) {
    if TRACK {
        visitor.transaction_options.if_depth += 1;
    }
    visitor.with_control_flow(|visitor| visitor.visit_statement(statement));
    if TRACK {
        visitor.transaction_options.if_depth -= 1;
    }
}

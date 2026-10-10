use super::{is_sql_tag, SqlTagNames};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{
    ArrayExpressionElement, AssignmentExpression, AssignmentTarget, BindingPattern, Expression,
    Program, TemplateLiteral, VariableDeclarator,
};
use oxc_ast_visit::{walk, Visit};
use std::collections::HashSet;

/// Upper bound on fixpoint passes over aliasing chains such as
/// `const a = sql``…``; const b = a; const c = cond ? b : sql``…``;`.
const MAX_ALIAS_PASSES: usize = 8;

/// Whether a trusted SQL tag's template interpolates another SQL fragment
/// rather than a parameterized value. Such a template is spliced from
/// several pieces at runtime, so its quasi text alone is not the statement
/// that executes: the fragment's own SQL (or one of several branch
/// fragments) replaces the `sql_placeholder_N` that would otherwise stand for
/// a bind value.
pub(super) fn interpolates_sql_fragment(
    template: &TemplateLiteral<'_>,
    is_shadowed: &mut impl FnMut(&str) -> bool,
    tags: &SqlTagNames,
) -> bool {
    template
        .expressions
        .iter()
        .any(|expr| is_sql_fragment(expr, is_shadowed, tags))
}

/// A trusted tagged template, a call on the trusted tag (`sql(...)`,
/// `sql.raw(...)`, `sql.join(...)`), a member call on another fragment
/// (`sql`…`.append(...)`), a binding known to hold one, or a conditional,
/// logical, sequence, or array expression that can evaluate to one.
fn is_sql_fragment(
    expr: &Expression<'_>,
    is_shadowed: &mut impl FnMut(&str) -> bool,
    tags: &SqlTagNames,
) -> bool {
    match unwrap_ts_wrappers(expr) {
        Expression::TaggedTemplateExpression(tagged) => is_sql_tag(&tagged.tag, is_shadowed, tags),
        Expression::CallExpression(call) => match unwrap_ts_wrappers(&call.callee) {
            Expression::StaticMemberExpression(member) => {
                is_sql_tag(&member.object, is_shadowed, tags)
                    || is_sql_fragment(&member.object, is_shadowed, tags)
            }
            callee => is_sql_tag(callee, is_shadowed, tags),
        },
        Expression::ConditionalExpression(conditional) => {
            is_sql_fragment(&conditional.consequent, is_shadowed, tags)
                || is_sql_fragment(&conditional.alternate, is_shadowed, tags)
        }
        Expression::LogicalExpression(logical) => {
            is_sql_fragment(&logical.left, is_shadowed, tags)
                || is_sql_fragment(&logical.right, is_shadowed, tags)
        }
        Expression::SequenceExpression(sequence) => sequence
            .expressions
            .last()
            .is_some_and(|last| is_sql_fragment(last, is_shadowed, tags)),
        Expression::ArrayExpression(array) => array.elements.iter().any(|element| match element {
            ArrayExpressionElement::SpreadElement(spread) => {
                is_sql_fragment(&spread.argument, is_shadowed, tags)
            }
            element => element
                .as_expression()
                .is_some_and(|expr| is_sql_fragment(expr, is_shadowed, tags)),
        }),
        Expression::Identifier(ident) => tags.fragments.contains(ident.name.as_str()),
        _ => false,
    }
}

/// Names bound or assigned anywhere in the file to a SQL fragment.
///
/// Deliberately scope-insensitive: a name that holds a fragment in one
/// function and a plain value in another is treated as a fragment in both,
/// so the composing template fails closed as `Dynamic` instead of reading a
/// spliced fragment as a bind value.
pub(super) fn collect_fragment_bindings(
    program: &Program<'_>,
    tags: &SqlTagNames,
) -> HashSet<String> {
    let mut current = SqlTagNames {
        imported: tags.imported.clone(),
        fragments: HashSet::new(),
    };
    for _ in 0..MAX_ALIAS_PASSES {
        let mut collector = FragmentBindings {
            tags: &current,
            found: HashSet::new(),
        };
        collector.visit_program(program);
        if collector.found.len() == current.fragments.len() {
            break;
        }
        current.fragments = collector.found;
    }
    current.fragments
}

struct FragmentBindings<'t> {
    tags: &'t SqlTagNames,
    found: HashSet<String>,
}

impl FragmentBindings<'_> {
    fn record(&mut self, name: &str, init: &Expression<'_>) {
        // Shadowing is unknown at file scope; spelling plus trusted imports
        // matches how the composing template's own tag is recognized.
        if is_sql_fragment(init, &mut |_| false, self.tags) {
            self.found.insert(name.to_string());
        }
    }
}

impl<'a> Visit<'a> for FragmentBindings<'_> {
    fn visit_variable_declarator(&mut self, declarator: &VariableDeclarator<'a>) {
        if let (BindingPattern::BindingIdentifier(ident), Some(init)) =
            (&declarator.id, &declarator.init)
        {
            self.record(ident.name.as_str(), init);
        }
        walk::walk_variable_declarator(self, declarator);
    }

    fn visit_assignment_expression(&mut self, assignment: &AssignmentExpression<'a>) {
        if let AssignmentTarget::AssignmentTargetIdentifier(ident) = &assignment.left {
            self.record(ident.name.as_str(), &assignment.right);
        }
        walk::walk_assignment_expression(self, assignment);
    }
}

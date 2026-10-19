use crate::queue::extract_helpers::{binding_name, literal_expr};
use crate::queue::extract_visitor::QueueVisitor;
use oxc_ast::ast::{
    ArrowFunctionExpression, Expression, FormalParameters, Function, Statement, SwitchStatement,
};
use oxc_ast_visit::{walk, Visit};

pub(crate) fn visit_function<'a>(
    visitor: &mut QueueVisitor<'a>,
    func: &Function<'a>,
    flags: oxc_syntax::scope::ScopeFlags,
) {
    let worker = enter_worker(visitor, func.span);
    let entered = has_job_param(&func.params);
    if entered {
        visitor.in_job += 1;
    }
    walk::walk_function(visitor, func, flags);
    if entered {
        visitor.in_job -= 1;
    }
    if worker {
        visitor.worker_stack.pop();
    }
}

pub(crate) fn visit_arrow<'a>(visitor: &mut QueueVisitor<'a>, arrow: &ArrowFunctionExpression<'a>) {
    let worker = enter_worker(visitor, arrow.span);
    let entered = has_job_param(&arrow.params);
    if entered {
        visitor.in_job += 1;
    }
    walk::walk_arrow_function_expression(visitor, arrow);
    if entered {
        visitor.in_job -= 1;
    }
    if worker {
        visitor.worker_stack.pop();
    }
}

fn enter_worker(visitor: &mut QueueVisitor<'_>, span: oxc_span::Span) -> bool {
    let Some(index) = visitor
        .processor_spans
        .iter()
        .rev()
        .find(|record| span.start >= record.0 && span.end <= record.1)
        .map(|record| record.2)
    else {
        return false;
    };
    visitor.worker_stack.push(index);
    true
}

pub(crate) fn visit_switch<'a>(visitor: &mut QueueVisitor<'a>, stmt: &SwitchStatement<'a>) {
    let tracks = is_job_name(&stmt.discriminant);
    visitor.visit_expression(&stmt.discriminant);
    for case in &stmt.cases {
        let label = tracks
            .then(|| case_label(visitor, case.test.as_ref()))
            .flatten();
        let previous = visitor.current_case.clone();
        if label.is_some() {
            visitor.current_case = label;
        }
        if let Some(test) = &case.test {
            visitor.visit_expression(test);
        }
        for statement in &case.consequent {
            visit_statement(visitor, statement);
        }
        visitor.current_case = previous;
    }
}

fn has_job_param(params: &FormalParameters<'_>) -> bool {
    params
        .items
        .iter()
        .any(|param| binding_name(&param.pattern).as_deref() == Some("job"))
}

fn is_job_name(expr: &Expression<'_>) -> bool {
    let Expression::StaticMemberExpression(member) = expr else {
        return false;
    };
    let Expression::Identifier(object) = &member.object else {
        return false;
    };
    object.name == "job" && member.property.name == "name"
}

fn case_label(visitor: &QueueVisitor<'_>, test: Option<&Expression<'_>>) -> Option<String> {
    literal_expr(test?, &visitor.const_strings)
}

fn visit_statement<'a>(visitor: &mut QueueVisitor<'a>, statement: &Statement<'a>) {
    visitor.visit_statement(statement);
}

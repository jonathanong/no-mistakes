use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{
    Argument, ArrayExpressionElement, ArrowFunctionExpression, AssignmentExpression,
    BindingPattern, CallExpression, Expression, ForOfStatement, FormalParameters, Function,
    TaggedTemplateExpression, UpdateExpression, VariableDeclarator, YieldExpression,
};
use oxc_ast_visit::{walk, Visit};
use oxc_syntax::scope::ScopeFlags;

/// Failed labels can change scalar guards before a later case or default runs.
/// The switch fallback discards finite facts instead of replaying those writes
/// on each entry/fallthrough path.
pub(super) fn contains_write(expression: &Expression<'_>) -> bool {
    let mut writes = Writes::default();
    writes.visit_expression(expression);
    writes.found
}

#[derive(Default)]
struct Writes {
    found: bool,
    consume_generators: bool,
}

impl Writes {
    fn visit_consumed_expression<'a>(&mut self, expression: &Expression<'a>) {
        let previous = self.consume_generators;
        self.consume_generators = true;
        self.visit_expression(expression);
        self.consume_generators = previous;
    }
}

impl<'a> Visit<'a> for Writes {
    fn visit_assignment_expression(&mut self, _: &AssignmentExpression<'a>) {
        // This includes object/array targets and writes through members.
        self.found = true;
    }

    fn visit_update_expression(&mut self, _: &UpdateExpression<'a>) {
        self.found = true;
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        // A member invocation or unknown argument consumer may advance an
        // iterator. Keep that conservative context through returned closures.
        match unwrap_ts_wrappers(&call.callee) {
            Expression::StaticMemberExpression(member) => {
                self.visit_consumed_expression(&member.object);
            }
            Expression::ComputedMemberExpression(member) => {
                self.visit_consumed_expression(&member.object);
                self.visit_expression(&member.expression);
            }
            Expression::PrivateFieldExpression(member) => {
                self.visit_consumed_expression(&member.object);
            }
            callee => self.visit_expression(callee),
        }
        self.visit_arguments(&call.arguments);
        if self.found {
            return;
        }
        match invocation_target(&call.callee) {
            Expression::ArrowFunctionExpression(arrow) => {
                walk::walk_arrow_function_expression(self, arrow);
            }
            Expression::FunctionExpression(function) => {
                if function.generator && !self.consume_generators {
                    // Defaults execute at iterator creation; its body does not.
                    self.visit_formal_parameters(&function.params);
                } else {
                    walk::walk_function(self, function, ScopeFlags::Function);
                }
            }
            _ => {}
        }
    }

    fn visit_argument(&mut self, argument: &Argument<'a>) {
        match argument {
            Argument::SpreadElement(spread) => self.visit_consumed_expression(&spread.argument),
            _ => self.visit_consumed_expression(argument.to_expression()),
        }
    }

    fn visit_arguments(&mut self, arguments: &oxc_allocator::Vec<'a, Argument<'a>>) {
        // Oxc's list walker dispatches expressions/spreads directly instead
        // of calling visit_argument, so retain consumption context here.
        for argument in arguments {
            self.visit_argument(argument);
        }
    }

    fn visit_tagged_template_expression(&mut self, expression: &TaggedTemplateExpression<'a>) {
        self.visit_expression(&expression.tag);
        // Tags receive interpolation values and may advance their iterators.
        // Ordinary template coercion does not consume a native iterator.
        for interpolation in &expression.quasi.expressions {
            self.visit_consumed_expression(interpolation);
        }
    }

    fn visit_array_expression_element(&mut self, element: &ArrayExpressionElement<'a>) {
        if let ArrayExpressionElement::SpreadElement(spread) = element {
            self.visit_consumed_expression(&spread.argument);
        } else {
            walk::walk_array_expression_element(self, element);
        }
    }

    fn visit_variable_declarator(&mut self, declarator: &VariableDeclarator<'a>) {
        self.visit_binding_pattern(&declarator.id);
        if let Some(initializer) = &declarator.init {
            // Stored iterators can be consumed through aliases later. Retain
            // the existing conservative body scan without tracing bindings.
            self.visit_consumed_expression(initializer);
        }
    }

    fn visit_formal_parameters(&mut self, parameters: &FormalParameters<'a>) {
        // Default iterator values are stored in parameter bindings too.
        let previous = self.consume_generators;
        self.consume_generators = true;
        walk::walk_formal_parameters(self, parameters);
        self.consume_generators = previous;
    }

    fn visit_binding_pattern(&mut self, pattern: &BindingPattern<'a>) {
        // Destructuring defaults can store an iterator independently of the
        // declaration initializer, including loop and catch bindings.
        let previous = self.consume_generators;
        self.consume_generators = true;
        walk::walk_binding_pattern(self, pattern);
        self.consume_generators = previous;
    }

    fn visit_for_of_statement(&mut self, statement: &ForOfStatement<'a>) {
        self.visit_for_statement_left(&statement.left);
        self.visit_consumed_expression(&statement.right);
        self.visit_statement(&statement.body);
    }

    fn visit_yield_expression(&mut self, expression: &YieldExpression<'a>) {
        match (&expression.argument, expression.delegate) {
            (Some(argument), true) => self.visit_consumed_expression(argument),
            _ => walk::walk_yield_expression(self, expression),
        }
    }

    // Creating or passing a callback does not execute its body.
    fn visit_function(&mut self, _: &Function<'a>, _: ScopeFlags) {}
    fn visit_arrow_function_expression(&mut self, _: &ArrowFunctionExpression<'a>) {}
}

fn invocation_target<'a>(expression: &'a Expression<'a>) -> &'a Expression<'a> {
    let expression = unwrap_ts_wrappers(expression);
    let receiver = match expression {
        Expression::StaticMemberExpression(member)
            if matches!(member.property.name.as_str(), "call" | "apply") =>
        {
            &member.object
        }
        Expression::ComputedMemberExpression(member)
            if matches!(unwrap_ts_wrappers(&member.expression), Expression::StringLiteral(property)
                if matches!(crate::ast::js_literal(property).as_ref(), "call" | "apply")) =>
        {
            &member.object
        }
        _ => return expression,
    };
    unwrap_ts_wrappers(receiver)
}

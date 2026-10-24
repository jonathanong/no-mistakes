use super::*;
use oxc_ast::ast::{
    ArrowFunctionExpression, ReturnStatement, TSAsExpression, TSSatisfiesExpression,
    TSTypeAssertion,
};
use oxc_span::GetSpan;

// A type at a runtime producer preserves uncertainty at the value expression.
// Erased declarations and return headers alone are never runtime obligations.
// Transparent wrappers share one value span, including typed asserted returns.
pub(super) fn collect(program: &Program<'_>, possible: &FxHashSet<Span>) -> FxHashSet<Span> {
    let mut visitor = Producers {
        possible,
        returns_executor: false,
        uses: FxHashSet::default(),
    };
    visitor.visit_program(program);
    visitor.uses
}

struct Producers<'s> {
    possible: &'s FxHashSet<Span>,
    returns_executor: bool,
    uses: FxHashSet<Span>,
}

impl Producers<'_> {
    fn value(&mut self, expression: &Expression<'_>, ty: &TSType<'_>) {
        if possible::contains(self.possible, ty) {
            self.uses
                .insert(span(expression.get_inner_expression().span()));
        }
    }
}

impl<'a> Visit<'a> for Producers<'_> {
    fn visit_ts_as_expression(&mut self, value: &TSAsExpression<'a>) {
        self.value(&value.expression, &value.type_annotation);
        walk::walk_ts_as_expression(self, value);
    }

    fn visit_ts_satisfies_expression(&mut self, value: &TSSatisfiesExpression<'a>) {
        self.value(&value.expression, &value.type_annotation);
        walk::walk_ts_satisfies_expression(self, value);
    }

    fn visit_ts_type_assertion(&mut self, value: &TSTypeAssertion<'a>) {
        self.value(&value.expression, &value.type_annotation);
        walk::walk_ts_type_assertion(self, value);
    }

    fn visit_function(&mut self, function: &Function<'a>, flags: oxc_syntax::scope::ScopeFlags) {
        let previous = self.returns_executor;
        self.returns_executor = function.body.is_some()
            && function.return_type.as_ref().is_some_and(|annotation| {
                possible::contains(self.possible, &annotation.type_annotation)
            });
        walk::walk_function(self, function, flags);
        self.returns_executor = previous;
    }

    fn visit_arrow_function_expression(&mut self, arrow: &ArrowFunctionExpression<'a>) {
        let previous = self.returns_executor;
        self.returns_executor = arrow.return_type.as_ref().is_some_and(|annotation| {
            possible::contains(self.possible, &annotation.type_annotation)
        });
        if self.returns_executor {
            if let Some(expression) = arrow.body.as_expression() {
                self.uses
                    .insert(span(expression.get_inner_expression().span()));
            }
        }
        walk::walk_arrow_function_expression(self, arrow);
        self.returns_executor = previous;
    }

    fn visit_return_statement(&mut self, statement: &ReturnStatement<'a>) {
        if self.returns_executor {
            if let Some(expression) = &statement.argument {
                self.uses
                    .insert(span(expression.get_inner_expression().span()));
            }
        }
        walk::walk_return_statement(self, statement);
    }

    fn visit_ts_type(&mut self, _: &TSType<'a>) {}
}

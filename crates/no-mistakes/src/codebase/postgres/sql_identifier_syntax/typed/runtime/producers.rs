use super::*;
use oxc_ast::ast::{
    ArrowFunctionExpression, ReturnStatement, TSAsExpression, TSSatisfiesExpression,
    TSTypeAssertion,
};
use oxc_span::GetSpan;
mod bindings;
mod methods;

#[derive(Default)]
pub(super) struct Produced {
    pub uses: FxHashSet<Span>,
    pub values: FxHashSet<usize>,
    pub transports: FxHashSet<Span>,
    pub functions: FxHashSet<usize>,
}
// Producer types supply negative receiver provenance, never literal SQL proof.
// Candidate spans are projected only at actual calls or opaque invocations.
// Transparent assertions share the underlying value span.
pub(super) fn collect(
    program: &Program<'_>,
    possible: &FxHashSet<Span>,
    ids: &FxHashMap<Span, usize>,
) -> Produced {
    let mut visitor = Producers {
        possible,
        bindings: bindings::Bindings::new(possible, ids),
        returns_executor: false,
        return_owner: None,
        uses: FxHashSet::default(),
    };
    visitor.visit_program(program);
    let mut produced = visitor.bindings.finish();
    produced.uses.extend(visitor.uses);
    produced
}

struct Producers<'s> {
    possible: &'s FxHashSet<Span>,
    bindings: bindings::Bindings<'s>,
    returns_executor: bool,
    return_owner: Option<usize>,
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
    fn visit_identifier_reference(&mut self, value: &oxc_ast::ast::IdentifierReference<'a>) {
        self.bindings.reference(span(value.span));
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        self.bindings.call(call);
        walk::walk_call_expression(self, call);
    }

    fn visit_variable_declarator(&mut self, variable: &oxc_ast::ast::VariableDeclarator<'a>) {
        self.bindings.variable(variable);
        walk::walk_variable_declarator(self, variable);
    }

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
        let owner = self.return_owner;
        self.return_owner = self.bindings.function(function);
        let previous = self.returns_executor;
        self.returns_executor = function.body.is_some()
            && function.return_type.as_ref().is_some_and(|annotation| {
                possible::contains(self.possible, &annotation.type_annotation)
            });
        walk::walk_function(self, function, flags);
        self.returns_executor = previous;
        self.return_owner = owner;
    }

    fn visit_arrow_function_expression(&mut self, arrow: &ArrowFunctionExpression<'a>) {
        let owner = self.return_owner;
        self.return_owner = self.bindings.owner(span(arrow.span));
        let previous = self.returns_executor;
        self.returns_executor = arrow.return_type.as_ref().is_some_and(|annotation| {
            possible::contains(self.possible, &annotation.type_annotation)
        });
        if let (Some(owner), Some(expression)) = (self.return_owner, arrow.body.as_expression()) {
            self.bindings.returned(owner, expression);
        }
        if self.returns_executor {
            if let Some(expression) = arrow.body.as_expression() {
                let at = span(expression.get_inner_expression().span());
                self.uses.insert(at);
            }
        }
        walk::walk_arrow_function_expression(self, arrow);
        self.returns_executor = previous;
        self.return_owner = owner;
    }

    fn visit_return_statement(&mut self, statement: &ReturnStatement<'a>) {
        if let (Some(owner), Some(expression)) = (self.return_owner, statement.argument.as_ref()) {
            self.bindings.returned(owner, expression);
        }
        if self.returns_executor {
            if let Some(expression) = &statement.argument {
                let at = span(expression.get_inner_expression().span());
                self.uses.insert(at);
            }
        }
        walk::walk_return_statement(self, statement);
    }

    fn visit_ts_type(&mut self, _: &TSType<'a>) {}
}

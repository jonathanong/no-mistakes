//! A parameter or catch binding named `Set` is not the intrinsic constructor.
use oxc_ast::ast;
use oxc_ast_visit::{walk, Visit};
use oxc_syntax::scope::ScopeFlags;

pub(in crate::codebase::postgres::embedded::walk) fn shadows_set(
    params: &ast::FormalParameters<'_>,
    body: &ast::FunctionBody<'_>,
) -> bool {
    let mut found = false;
    for pattern in params
        .items
        .iter()
        .map(|item| &item.pattern)
        .chain(params.rest.as_ref().map(|rest| &rest.rest.argument))
    {
        super::super::resolve::for_each_bound_name(pattern, &mut |name| {
            found |= name == "Set";
        });
    }
    let mut catches = CatchSet { found };
    catches.visit_function_body(body);
    catches.found
}

struct CatchSet {
    found: bool,
}

impl<'a> Visit<'a> for CatchSet {
    fn visit_function(&mut self, _function: &ast::Function<'a>, _flags: ScopeFlags) {}

    fn visit_arrow_function_expression(&mut self, _arrow: &ast::ArrowFunctionExpression<'a>) {}

    fn visit_class(&mut self, _class: &ast::Class<'a>) {}

    fn visit_catch_clause(&mut self, clause: &ast::CatchClause<'a>) {
        if let Some(param) = &clause.param {
            super::super::resolve::for_each_bound_name(&param.pattern, &mut |name| {
                self.found |= name == "Set";
            });
        }
        walk::walk_catch_clause(self, clause);
    }
}

#[cfg(test)]
mod tests {
    use super::shadows_set;
    use oxc_ast::ast::{Function, Statement};

    fn function<'a>(program: &'a oxc_ast::ast::Program<'a>, name: &str) -> &'a Function<'a> {
        program
            .body
            .iter()
            .find_map(|statement| {
                let Statement::FunctionDeclaration(function) = statement else {
                    return None;
                };
                function
                    .id
                    .as_ref()
                    .is_some_and(|id| id.name == name)
                    .then_some(function.as_ref())
            })
            .unwrap_or_else(|| panic!("missing {name}"))
    }

    #[test]
    fn saved_fixture_rejects_set_parameters_and_catches() {
        let source = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../test-cases/rules/postgres-no-unbounded-sql-identifier/fixture/fail-shadowed-set-guard/src/query.ts"
        ));
        crate::ast::with_program(std::path::Path::new("query.ts"), source, |program, _| {
            for (name, shadowed) in [("trusted", false), ("shadowed", true), ("caught", true)] {
                let function = function(program, name);
                let body = function.body.as_ref().unwrap();
                assert_eq!(shadows_set(&function.params, body), shadowed, "{name}");
            }
        })
        .unwrap();
    }
}

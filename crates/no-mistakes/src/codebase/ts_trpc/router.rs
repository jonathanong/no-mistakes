use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Argument, CallExpression, Expression, ObjectExpression, ObjectPropertyKind, Program,
};
use oxc_ast_visit::{walk, Visit};
use oxc_span::SourceType;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TrpcRouterFacts {
    pub procedures: Vec<String>,
}

#[cfg(test)]
pub fn extract_trpc_router(source: &str) -> TrpcRouterFacts {
    let allocator = Allocator::default();
    let ret = crate::ast::parse(
        std::path::Path::new("trpc-router.ts"),
        &allocator,
        source,
        SourceType::ts(),
    );
    extract_trpc_router_from_program(&ret.program)
}

pub fn extract_trpc_router_from_program(program: &Program<'_>) -> TrpcRouterFacts {
    let mut visitor = RouterVisitor {
        procedures: Vec::new(),
    };
    visitor.visit_program(program);
    visitor.procedures.sort();
    visitor.procedures.dedup();
    TrpcRouterFacts {
        procedures: visitor.procedures,
    }
}

struct RouterVisitor {
    procedures: Vec<String>,
}

impl<'a> Visit<'a> for RouterVisitor {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if is_router_callee(&call.callee) {
            if let Some(Expression::ObjectExpression(object)) =
                call.arguments.first().and_then(Argument::as_expression)
            {
                collect_procedures(object, "", &mut self.procedures);
            }
        }
        walk::walk_call_expression(self, call);
    }
}

fn is_router_callee(callee: &Expression<'_>) -> bool {
    match callee {
        Expression::Identifier(id) => matches!(
            id.name.as_str(),
            "router" | "createTRPCRouter" | "createTrpcRouter"
        ),
        Expression::StaticMemberExpression(member) => member.property.name.as_str() == "router",
        _ => false,
    }
}

fn collect_procedures(object: &ObjectExpression<'_>, prefix: &str, out: &mut Vec<String>) {
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        let Some(key) = property.key.static_name() else {
            continue;
        };
        let path = if prefix.is_empty() {
            key.to_string()
        } else {
            format!("{prefix}.{key}")
        };
        match &property.value {
            Expression::ObjectExpression(nested) => collect_procedures(nested, &path, out),
            Expression::CallExpression(call) if is_procedure_definition(call) => out.push(path),
            _ => {}
        }
    }
}

fn is_procedure_definition(call: &CallExpression<'_>) -> bool {
    match &call.callee {
        Expression::StaticMemberExpression(member) => {
            matches!(member.property.name.as_str(), "query" | "mutation")
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests;

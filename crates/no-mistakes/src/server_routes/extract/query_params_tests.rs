use super::*;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;

fn params_from_source(source: &str) -> Vec<String> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let mut params = BTreeSet::new();
    for statement in &parsed.program.body {
        collect_query_params_from_statement(statement, &mut params);
    }
    params.into_iter().collect()
}

#[test]
fn query_param_walker_covers_statement_and_expression_shapes() {
    let params = params_from_source(
        r#"
        const { first = "x", second: renamed } = req.query;
        const { [dynamicKey]: computed = req.query.assignedPattern } = req.query;
        const [] = req.query;
        const condition = req.query.a && req.query.b ? req.query.c : req.query.d;
        const computedDynamic = req.query[dynamicKey()];
        const computedOtherObject = other[req.query.computedOtherObject];
        const sequence = (req.query.sequence, req.query.afterSequence);
        const object = { nested: req.query.object };
        const array = [req.query.array];
        const paren = (req.query.paren);
        const cast = req.query.cast as string;
        const assertion = <string>req.query.assertion;
        const nonNull = req.query.nonNull!;
        const satisfies = req.query.satisfies satisfies string;
        req.query("call");
        req.queries("calls");
        c.req.query("hono");
        db.query("select * from users");
        ({}).query("objectQuery");
        req.query(dynamic);
        req.get("ignored");
        new URLSearchParams("?url=value").get("url");
        if (req.query.ifTest) { req.query.ifBody; } else { req.query.elseBody; }
        for (let i = Number(req.query.forInit); i < Number(req.query.forTest); i += 1) {
            req.query.forBody;
        }
        for (; req.query.forNoInitTest; ) { break; }
        while (req.query.whileTest) { break; }
        for (const item of [req.query.forOf]) { item; }
        for (const key in { value: req.query.forIn }) { key; }
        switch (req.query.switchTest) { case "x": req.query.switchCase; }
        try { req.query.tryBody; } catch { req.query.catchBody; } finally { req.query.finallyBody; }
        function nested() { return req.query.functionBody; }
        const nestedFunctionExpression = function () { return req.query.functionExpressionBody; };
        export function exported() { return req.query.exportedFunction; }
        export const exportedValue = req.query.exportedValue;
        const localExported = req.query.exportSpecifierLocal;
        export { localExported };
        export class ExportedClass {}
        async function awaits() { await req.query.awaited; }
        "#,
    );

    for param in [
        "afterSequence",
        "array",
        "assertion",
        "awaited",
        "call",
        "calls",
        "cast",
        "catchBody",
        "computedOtherObject",
        "elseBody",
        "exportedFunction",
        "exportedValue",
        "finallyBody",
        "first",
        "forBody",
        "forIn",
        "forInit",
        "forNoInitTest",
        "forOf",
        "functionBody",
        "functionExpressionBody",
        "hono",
        "ifBody",
        "ifTest",
        "exportSpecifierLocal",
        "nonNull",
        "object",
        "paren",
        "satisfies",
        "second",
        "switchCase",
        "switchTest",
        "tryBody",
        "url",
        "whileTest",
    ] {
        assert!(
            params.iter().any(|value| value == param),
            "{param}: {params:?}"
        );
    }
    assert!(!params.iter().any(|value| value == "ignored"));
    assert!(!params.iter().any(|value| value == "select * from users"));
    assert!(!params.iter().any(|value| value == "objectQuery"));
}

#[test]
fn query_param_arg_walker_handles_function_expressions_and_non_handlers() {
    let allocator = Allocator::default();
    let parsed = Parser::new(
        &allocator,
        r#"
        app.get("/fn", function () { return req.query.functionExpression; });
        app.get("/noop", "not a function");
        app.get("/num", 123);
        app.get("/spread", ...handlers);
        "#,
        SourceType::ts(),
    )
    .parse();
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let mut params = BTreeSet::new();
    let visitor = ServerRouteVisitor::new(std::path::Path::new("routes.ts"), "");

    for statement in &parsed.program.body {
        if let Statement::ExpressionStatement(statement) = statement {
            if let Expression::CallExpression(call) = &statement.expression {
                for arg in &call.arguments {
                    visitor.collect_query_params_from_arg(arg, &mut params);
                }
            }
        }
    }

    assert!(params.contains("functionExpression"));
    collect_query_params_from_optional_function_body(None, &mut params);
}

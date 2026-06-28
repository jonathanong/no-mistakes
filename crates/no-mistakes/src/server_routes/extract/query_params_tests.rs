use super::*;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use std::collections::HashMap;

fn params_from_source(source: &str) -> Vec<String> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let mut params = BTreeSet::new();
    let named_handlers = HashMap::new();
    for statement in &parsed.program.body {
        collect_query_params_from_statement(statement, &mut params, &named_handlers);
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
        req?.query?.("optionalCall");
        c.req.query("honoCall");
        req.context.query("requestContextCall");
        context.request.query("contextCall");
        context?.request?.query?.("optionalContextCall");
        context.req.query("contextReqCall");
        context.deep.req.query("ignoredNestedRequest");
        unrelated.req.query("ignoredMemberObject");
        req.query(dynamic);
        unrelated.query("ignoredCall");
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
        "honoCall",
        "ifBody",
        "ifTest",
        "exportSpecifierLocal",
        "nonNull",
        "object",
        "paren",
        "requestContextCall",
        "satisfies",
        "second",
        "switchCase",
        "switchTest",
        "tryBody",
        "url",
        "whileTest",
        "contextCall",
        "contextReqCall",
    ] {
        assert!(
            params.iter().any(|value| value == param),
            "{param}: {params:?}"
        );
    }
    assert!(!params.iter().any(|value| value == "ignored"));
    assert!(!params.iter().any(|value| value == "ignoredCall"));
    assert!(!params.iter().any(|value| value == "ignoredMemberObject"));
    assert!(!params.iter().any(|value| value == "ignoredNestedRequest"));
}

fn expression_matches_request_object(source: &str) -> bool {
    let allocator = Allocator::default();
    let source = format!("const value = {source};");
    let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let Statement::VariableDeclaration(declaration) = &parsed.program.body[0] else {
        panic!("expected variable declaration");
    };
    let expression = declaration.declarations[0].init.as_ref().unwrap();
    is_request_query_object(expression)
}

fn expression_matches_request_object_at_nesting(source: &str, nesting: u8) -> bool {
    let allocator = Allocator::default();
    let source = format!("const value = {source};");
    let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let Statement::VariableDeclaration(declaration) = &parsed.program.body[0] else {
        panic!("expected variable declaration");
    };
    let expression = declaration.declarations[0].init.as_ref().unwrap();
    is_request_object_expr(expression, nesting)
}

#[test]
fn request_query_object_detection_handles_optional_and_nested_members() {
    assert!(expression_matches_request_object("context?.request"));
    assert!(!expression_matches_request_object("context?.[dynamicKey]"));
    assert!(!expression_matches_request_object("context?.other"));
    assert!(!expression_matches_request_object(
        "context?.request?.query()"
    ));
    assert!(expression_matches_request_object("context.req"));
    assert!(expression_matches_request_object_at_nesting(
        "context.req",
        1
    ));
    assert!(!expression_matches_request_object_at_nesting(
        "context.req",
        2
    ));
    assert!(!expression_matches_request_object_at_nesting("call()", 1));
    assert!(!expression_matches_request_object("context.deep.req"));
    assert!(!expression_matches_request_object(
        "context.req.ctx.request"
    ));
    assert!(!expression_matches_request_object("call().req"));
    assert!(!expression_matches_request_object("unrelated.req"));
    assert!(!expression_matches_request_object("123"));
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
    let named_handlers = HashMap::new();

    for statement in &parsed.program.body {
        if let Statement::ExpressionStatement(statement) = statement {
            if let Expression::CallExpression(call) = &statement.expression {
                for arg in &call.arguments {
                    collect_query_params_from_arg(arg, &mut params, &named_handlers);
                }
            }
        }
    }

    assert!(params.contains("functionExpression"));
    collect_query_params_from_optional_function_body(None, &mut params, &named_handlers);
}

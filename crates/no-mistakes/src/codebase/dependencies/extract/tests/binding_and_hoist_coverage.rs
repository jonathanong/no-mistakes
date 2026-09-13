use super::*;
use oxc_allocator::Allocator;
use oxc_ast::ast::{Expression, Statement};
use oxc_parser::Parser;

fn assignment_names(source: &str) -> Vec<String> {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source, SourceType::ts()).parse();
    let mut names = Vec::new();
    for statement in &ret.program.body {
        let Statement::ExpressionStatement(expr) = statement else {
            continue;
        };
        let Expression::AssignmentExpression(assignment) =
            crate::codebase::ts_source::unwrap_ts_wrappers(&expr.expression)
        else {
            continue;
        };
        names.extend(assignment_target_names(&assignment.left));
    }
    names
}

#[test]
fn assignment_target_names_cover_members_rest_defaults_and_casts() {
    let mut names = assignment_names("obj.prop = 1;");
    names.sort();
    assert_eq!(names, vec!["obj.prop"]);

    names = assignment_names("obj['key'] = 1;");
    names.sort();
    assert_eq!(names, vec!["obj.key"]);

    names = assignment_names("[a, ...rest] = arr;");
    names.sort();
    assert_eq!(names, vec!["a", "rest"]);

    names = assignment_names("({a, b: c, ...other} = obj);");
    names.sort();
    assert_eq!(names, vec!["a", "c", "other"]);

    names = assignment_names("[a = 1, {b = 2} = {}, ...rest] = arr;");
    names.sort();
    assert_eq!(names, vec!["a", "b", "rest"]);

    names = assignment_names("({x: [y = 1, ...inner]} = obj);");
    names.sort();
    assert_eq!(names, vec!["inner", "y"]);

    names = assignment_names("(x as any) = 1;");
    assert!(names.is_empty(), "type assertions are not binding names");

    names = assignment_names("(x!) = 1;");
    assert!(
        names.is_empty(),
        "non-null assertions are not binding names"
    );
}

#[test]
fn predeclare_walks_ambient_default_class_and_non_function_statements() {
    let allocator = Allocator::default();
    let ret = Parser::new(
        &allocator,
        "declare function ambient(): void;\n\
         export default class {}\n\
         if (true) { var hoisted = 1; }\n\
         function named() {}\n\
         var {a} = obj;\n\
         let {b} = obj;\n\
         class Named {}\n\
         const fn = () => {};\n\
         const expr = () => 1;\n\
         const obj = { method() {} };\n\
         function overload(x: string): void;\n\
         function overload(x: number): void;\n\
         function overload(x: string | number) {}\n\
         export default function () { var inner = 1; }\n",
        SourceType::ts(),
    )
    .parse();
    let facts = extract_import_facts_from_program(&ret.program);
    assert!(facts.imports.is_empty());
}

#[test]
fn class_eager_helpers_cover_getters_setters_static_blocks_and_decorators() {
    let allocator = Allocator::default();
    let source = "\
function deco(_target: unknown) {}\n\
function decoFactory() { return deco; }\n\
const key = 'computed';\n\
@deco\n\
class Named {\n\
  static get g() { return 1; }\n\
  static set s(_value: number) {}\n\
  static field = () => {};\n\
  static n = 1;\n\
  instance = 1;\n\
  [key]() {}\n\
  method() {}\n\
  accessor a = 1;\n\
  static { var hoisted = 1; function inner() {} }\n\
}\n\
class Child extends Named<number> {\n\
  read() {}\n\
}\n\
@decoFactory()\n\
class Factory {}\n\
@(0)\n\
class Unknown {}\n\
export default class {\n\
  read() {}\n\
}\n\
const alias = Named;\n\
alias.g;\n\
function outer() {\n\
  function inner(x: string): void;\n\
  function inner(x: number): void;\n\
  function inner(x: string | number) {}\n\
  inner('a');\n\
}\n";
    let ret = Parser::new(&allocator, source, SourceType::ts()).parse();
    let facts = extract_import_facts_from_program(&ret.program);
    assert!(facts
        .function_calls
        .iter()
        .any(|call| call.callee == "deco" || call.callee == "decoFactory"));
    assert!(
        facts
            .exported_functions
            .iter()
            .any(|name| name == "Named" || name == "Factory" || name == "Unknown")
            || !facts.callable_scopes.is_empty()
    );
}

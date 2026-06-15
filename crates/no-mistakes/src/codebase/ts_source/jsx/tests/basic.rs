use super::*;
use oxc_allocator::Allocator;
use oxc_ast::ast::JSXElementName;
use oxc_parser::Parser;
use oxc_span::SourceType;
use std::path::PathBuf;

fn parse<'a>(allocator: &'a Allocator, source: &'a str) -> oxc_ast::ast::Program<'a> {
    Parser::new(allocator, source, SourceType::tsx())
        .parse()
        .program
}

fn fixture_source(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/ast-snippets/ts-source/fixture")
        .join(name);
    std::fs::read_to_string(path).expect("fixture source must be readable")
}

#[test]
fn detects_jsx_presence() {
    let allocator = Allocator::default();
    assert!(program_contains_jsx(&parse(
        &allocator,
        "export const X = () => <div>hi</div>;",
    )));
}

#[test]
fn detects_no_jsx_in_pure_ts() {
    let allocator = Allocator::default();
    assert!(!program_contains_jsx(&parse(
        &allocator,
        "export const add = (a: number, b: number) => a + b;",
    )));
}

#[test]
fn detects_fragment_as_jsx() {
    let allocator = Allocator::default();
    assert!(program_contains_jsx(&parse(
        &allocator,
        "export const X = () => <>hi</>;",
    )));
}

#[test]
fn walk_program_visits_nested_jsx_openings() {
    struct Collect(Vec<String>);
    impl Visitor for Collect {
        fn visit_jsx_opening(&mut self, opening: &JSXOpeningElement) {
            if let JSXElementName::Identifier(id) = &opening.name {
                self.0.push(id.name.to_string());
            }
        }
    }

    let allocator = Allocator::default();
    let program = parse(
        &allocator,
        r#"
        export const X = () => (
            <div>
                <span>inner</span>
                {true && <img src="/x.png" />}
            </div>
        );
        "#,
    );
    let mut c = Collect(Vec::new());
    walk_program(&program, &mut c);
    assert_eq!(c.0, vec!["div", "span", "img"]);
}

#[test]
fn jsx_identifier_name_returns_tag() {
    struct First(Vec<String>);
    impl Visitor for First {
        fn visit_jsx_opening(&mut self, opening: &JSXOpeningElement) {
            if let Some(name) = jsx_identifier_name(opening) {
                self.0.push(name.to_string());
            }
        }
    }

    let allocator = Allocator::default();
    let program = parse(&allocator, "const x = <><div /><Link href=\"/a\" /></>;");
    let mut f = First(Vec::new());
    walk_program(&program, &mut f);
    assert_eq!(f.0, vec!["div", "Link"]);
}

#[test]
fn find_string_attr_reads_string_literal_and_expression_container() {
    struct Grab {
        target: Option<String>,
        rel: Option<String>,
        dynamic: Option<bool>,
    }
    impl Visitor for Grab {
        fn visit_jsx_opening(&mut self, opening: &JSXOpeningElement) {
            if let Some((_, Some(v))) = find_string_attr(opening, "target") {
                self.target = Some(v.to_string());
            }
            if let Some((_, Some(v))) = find_string_attr(opening, "rel") {
                self.rel = Some(v.to_string());
            }
            if let Some((present, value)) = find_string_attr(opening, "dynamic") {
                // boolean shorthand -> (true, None)
                self.dynamic = Some(present && value.is_none());
            }
        }
    }

    let allocator = Allocator::default();
    let program = parse(
        &allocator,
        r#"const x = <a target="_blank" rel={"nofollow"} dynamic />;"#,
    );
    let mut g = Grab {
        target: None,
        rel: None,
        dynamic: None,
    };
    walk_program(&program, &mut g);
    assert_eq!(g.target.as_deref(), Some("_blank"));
    assert_eq!(g.rel.as_deref(), Some("nofollow"));
    assert_eq!(g.dynamic, Some(true));
}

#[test]
fn default_visitor_hooks_and_non_identifier_attrs_are_exercised() {
    struct Noop;
    impl Visitor for Noop {}

    let allocator = Allocator::default();
    let program = parse(
        &allocator,
        r#"
        import x from "x";
        const value = <ns:tag ns:attr="x" dynamic={value} {...props}>{/* empty */}</ns:tag>;
        "#,
    );
    let mut noop = Noop;
    walk_program(&program, &mut noop);

    struct Grab {
        tag: Option<String>,
        dynamic: Option<(bool, Option<String>)>,
        missing: bool,
    }
    impl Visitor for Grab {
        fn visit_jsx_opening(&mut self, opening: &JSXOpeningElement) {
            self.tag = jsx_identifier_name(opening).map(str::to_string);
            self.dynamic = find_string_attr(opening, "dynamic")
                .map(|(present, value)| (present, value.map(str::to_string)));
            self.missing = find_string_attr(opening, "missing").is_none();
        }
    }
    let mut grab = Grab {
        tag: Some("unchanged".to_string()),
        dynamic: None,
        missing: false,
    };
    walk_program(&program, &mut grab);

    assert_eq!(grab.tag, None);
    assert_eq!(grab.dynamic, Some((true, None)));
    assert!(grab.missing);
}

#[test]
fn jsx_attr_helpers_cover_empty_and_non_string_values() {
    struct Grab {
        empty: Option<(bool, Option<String>)>,
        number: Option<(bool, Option<String>)>,
        element: Option<(bool, Option<String>)>,
        spread_seen: bool,
    }
    impl Visitor for Grab {
        fn visit_jsx_opening(&mut self, opening: &JSXOpeningElement) {
            self.empty = find_string_attr(opening, "empty")
                .map(|(present, value)| (present, value.map(str::to_string)));
            self.number = find_string_attr(opening, "number")
                .map(|(present, value)| (present, value.map(str::to_string)));
            self.element = find_string_attr(opening, "element")
                .map(|(present, value)| (present, value.map(str::to_string)));
            self.spread_seen |= find_string_attr(opening, "spread").is_none();
        }
    }

    let allocator = Allocator::default();
    let program = parse(
        &allocator,
        r#"const value = <Link empty={} number={1} element=<span /> {...props} />;"#,
    );
    let mut grab = Grab {
        empty: None,
        number: None,
        element: None,
        spread_seen: false,
    };
    walk_program(&program, &mut grab);
    assert_eq!(grab.empty, Some((true, None)));
    assert_eq!(grab.number, Some((true, None)));
    assert_eq!(grab.element, Some((true, None)));
    assert!(grab.spread_seen);
}

#[test]
fn visit_expression_hits_assignments_inside_jsx_handlers() {
    struct CountAssigns(usize);
    impl Visitor for CountAssigns {
        fn visit_expression(&mut self, expr: &Expression) {
            if matches!(expr, Expression::AssignmentExpression(_)) {
                self.0 += 1;
            }
        }
    }

    let allocator = Allocator::default();
    let program = parse(
        &allocator,
        r#"
        export const X = () => (
            <button onClick={() => { window.location.href = "/x"; }}>click</button>
        );
        "#,
    );
    let mut c = CountAssigns(0);
    walk_program(&program, &mut c);
    assert_eq!(c.0, 1);
}

#[test]
fn visits_import_declarations() {
    struct Imports(Vec<String>);
    impl Visitor for Imports {
        fn visit_import(&mut self, import: &oxc_ast::ast::ImportDeclaration) {
            self.0.push(import.source.value.to_string());
        }
    }

    let allocator = Allocator::default();
    let program = parse(
        &allocator,
        "import Link from \"next/link\"; import { a } from \"@/lib\";",
    );
    let mut i = Imports(Vec::new());
    walk_program(&program, &mut i);
    assert_eq!(i.0, vec!["next/link", "@/lib"]);
}

#[test]
fn visit_expression_hits_assignment_inside_spread_attr() {
    struct CountAssigns(usize);
    impl Visitor for CountAssigns {
        fn visit_expression(&mut self, expr: &Expression) {
            if matches!(expr, Expression::AssignmentExpression(_)) {
                self.0 += 1;
            }
        }
    }

    let allocator = Allocator::default();
    let program = parse(
        &allocator,
        r#"
        export const X = () => (
            <Comp {...{ onClick: () => { window.location.href = "/x"; } }} />
        );
        "#,
    );
    let mut c = CountAssigns(0);
    walk_program(&program, &mut c);
    assert_eq!(c.0, 1);
}

#[test]
fn walker_visits_statement_expression_and_jsx_variants_from_fixture() {
    struct Counts {
        expressions: usize,
        jsx_openings: usize,
        jsx_containers: usize,
        imports: usize,
    }
    impl Visitor for Counts {
        fn visit_import(&mut self, _import: &oxc_ast::ast::ImportDeclaration) {
            self.imports += 1;
        }

        fn visit_expression(&mut self, _expr: &Expression) {
            self.expressions += 1;
        }

        fn visit_jsx_opening(&mut self, _opening: &JSXOpeningElement) {
            self.jsx_openings += 1;
        }

        fn visit_jsx_expression_container(&mut self, _expr: &JSXExpression, _span_start: u32) {
            self.jsx_containers += 1;
        }
    }

    let source = fixture_source("jsx-walk-all.tsx");
    let allocator = Allocator::default();
    let program = parse(&allocator, &source);
    let mut counts = Counts {
        expressions: 0,
        jsx_openings: 0,
        jsx_containers: 0,
        imports: 0,
    };
    walk_program(&program, &mut counts);

    assert_eq!(counts.imports, 1);
    assert!(counts.expressions > 70, "{:?}", counts.expressions);
    assert!(counts.jsx_openings >= 8, "{:?}", counts.jsx_openings);
    assert!(counts.jsx_containers >= 7, "{:?}", counts.jsx_containers);
}

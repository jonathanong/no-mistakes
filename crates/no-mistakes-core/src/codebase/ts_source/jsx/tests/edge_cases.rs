#[test]
fn program_contains_jsx_walks_fixture_statement_shapes() {
    let source = fixture_source("jsx-walk-all.tsx");
    let allocator = Allocator::default();
    let program = parse(&allocator, &source);
    assert!(program_contains_jsx(&program));
}

#[test]
fn program_contains_jsx_walks_non_jsx_fixture_statement_shapes() {
    let source = fixture_source("no-jsx-walk-all.ts");
    let allocator = Allocator::default();
    let program = parse(&allocator, &source);
    assert!(!program_contains_jsx(&program));
}

#[test]
fn optional_walk_helpers_visit_present_nodes() {
    struct Count(usize);
    impl Visitor for Count {
        fn visit_expression(&mut self, _expr: &Expression) {
            self.0 += 1;
        }
    }

    let source = fixture_source("no-jsx-walk-all.ts");
    let allocator = Allocator::default();
    let program = parse(&allocator, &source);
    let mut count = Count(0);

    let if_stmt = program
        .body
        .iter()
        .find_map(|stmt| match stmt {
            Statement::IfStatement(if_stmt) => Some(if_stmt),
            _ => None,
        })
        .expect("fixture must contain an if statement");
    walk_optional_statement(if_stmt.alternate.as_ref(), &mut count);

    let export = program
        .body
        .iter()
        .find_map(|stmt| match stmt {
            Statement::ExportNamedDeclaration(export) => Some(export),
            _ => None,
        })
        .expect("fixture must contain a named export");
    walk_optional_declaration(export.declaration.as_ref(), &mut count);

    let var_decl = program
        .body
        .iter()
        .find_map(|stmt| match stmt {
            Statement::VariableDeclaration(var_decl) => Some(var_decl),
            _ => None,
        })
        .expect("fixture must contain a variable declaration");
    walk_optional_expression(var_decl.declarations[0].init.as_ref(), &mut count);

    assert!(count.0 > 0);
}

#[test]
fn walker_visits_default_exports_and_edge_expression_shapes() {
    struct Counts {
        expressions: usize,
        openings: usize,
        containers: usize,
    }
    impl Visitor for Counts {
        fn visit_expression(&mut self, _expr: &Expression) {
            self.expressions += 1;
        }

        fn visit_jsx_opening(&mut self, _opening: &JSXOpeningElement) {
            self.openings += 1;
        }

        fn visit_jsx_expression_container(&mut self, _expr: &JSXExpression, _span_start: u32) {
            self.containers += 1;
        }
    }

    for source in [
        r#"
        export default function DefaultFn() {
          return <div>{value}</div>;
        }
        "#,
        r#"
        export default class DefaultClass {
          render() {
            return <section>{items?.[key]}</section>;
          }
        }
        "#,
        r#"
        export default (ready ? <A attr={value as string} /> : <B {...props}>{...children}</B>);
        "#,
        r#"
        const value = (
          (target["key"] = call(...args, ...more)),
          target?.[key],
          tag`value-${expr}`,
          <ns:Tag attr={}>{value}</ns:Tag>
        );
        "#,
    ] {
        let allocator = Allocator::default();
        let program = parse(&allocator, source);
        let mut counts = Counts {
            expressions: 0,
            openings: 0,
            containers: 0,
        };
        walk_program(&program, &mut counts);
        assert!(counts.expressions > 0);
    }
}

#[test]
fn walker_directly_exercises_sparse_statement_and_expression_branches() {
    struct Count(usize);
    impl Visitor for Count {
        fn visit_expression(&mut self, _expr: &Expression) {
            self.0 += 1;
        }
    }

    let source = r#"
declare function ambient(): void;
declare class Ambient { render(): void; }

for (;;) {
  break;
}

try {
  value;
} catch {
  caught;
}

try {
  value;
} finally {
  cleaned;
}

switch (kind) {
  default:
    fallback;
}

function* gen() {
  yield;
  yield value;
}

class Mixed {
  field = ignored;
  method() {
    this.value;
  }
}

export function declared() {
  return <Declared />;
}

export class DeclaredClass {
  field = ignored;
  method() {
    return <DeclaredClassView />;
  }
}

	export default class DefaultWithField {
	  field = ignored;
	  method() {
	    return <DefaultClassView />;
	  }
	}

	export default function DefaultFunction() {
	  <DefaultFunctionView />;
	}

	export function NamedFunction() {
	  <NamedFunctionView />;
	}

	export class NamedClass {
	  method() {
	    <NamedClassView />;
	  }
	}

	export enum ExportedEnum {
	  A,
	}

	const chainCall = target?.method?.(arg);
	const chainMember = target?.[key];
	const assignment = (target[key] = value);
	const update = target[key]++;
	const optionalStatic = target?.prop;
	const tagged = tag`literal`;
	const array = [first, , ...rest];
	const object = { a: first, ...rest };
	const typed = (await (value as string)!) satisfies unknown;
	const fnExpr = function () {
	  nested;
	};
	"#;
    let allocator = Allocator::default();
    let program = parse(&allocator, source);
    let mut count = Count(0);
    walk_program(&program, &mut count);
    assert!(count.0 > 20, "visited {} expressions", count.0);
}

#[test]
fn walker_visits_remaining_expression_shapes() {
    struct Count(usize);
    impl Visitor for Count {
        fn visit_expression(&mut self, _expr: &Expression) {
            self.0 += 1;
        }
    }
    let allocator = Allocator::default();
    let program = Parser::new(
        &allocator,
        "export const value = (!flag, <string>raw, fn(1, ...args), new Thing(1, ...args), maybe?.method?.(1, ...args), maybe?.plain, function () { inner; });",
        SourceType::ts(),
    ).parse().program;
    let mut count = Count(0);
    walk_program(&program, &mut count);
    assert!(count.0 > 20, "visited {} expressions", count.0);
}

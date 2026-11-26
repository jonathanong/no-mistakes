use super::{property_types, value_kind_from_type};
use crate::builtin_receivers::names::{Names, ValueKind};
use oxc_ast::ast::Statement;
use std::path::Path;

#[test]
fn type_values_cover_qualified_unparameterized_and_property_signatures() {
    let source = r#"
        declare const q: NS.Item;
        declare const raw: Array;
        declare const store: AsyncLocalStorage<number>;
        declare const obj: { [k: string]: number; [sym]: string; name: string; run(): void };
        q.trim();
        raw.map((x: number) => x);
        store.getStore();
        obj.name.trim();
    "#;
    let proven = crate::ast::with_program(Path::new("types.ts"), source, |program, _| {
        crate::builtin_receivers::collect(program)
    })
    .unwrap();
    assert!(!proven.calls.is_empty());
}

#[test]
fn direct_type_value_and_property_type_edge_cases() {
    let source = r#"
        type Qualified = NS.Item;
        type Storage = AsyncLocalStorage<number>;
        type RawArray = Array;
        type Props = { name: string; "\uD800": string; [sym]: number; run(): void };
    "#;
    crate::ast::with_program(Path::new("direct-types.ts"), source, |program, _| {
        let mut results = Vec::new();
        let names = Names::default();
        let mut props = Vec::new();
        for stmt in &program.body {
            if let Statement::TSTypeAliasDeclaration(alias) = stmt {
                results.push((
                    alias.id.name.as_str().to_owned(),
                    value_kind_from_type(&alias.type_annotation),
                ));
                if alias.id.name == "Props" {
                    props = property_types(&names, &alias.type_annotation);
                }
            }
        }
        assert_eq!(
            results,
            vec![
                ("Qualified".to_owned(), None),
                ("Storage".to_owned(), Some(ValueKind::Builtin)),
                ("RawArray".to_owned(), None),
                ("Props".to_owned(), None),
            ]
        );
        assert_eq!(props, vec![("name".to_owned(), Some(ValueKind::String))]);
    })
    .unwrap();
}

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

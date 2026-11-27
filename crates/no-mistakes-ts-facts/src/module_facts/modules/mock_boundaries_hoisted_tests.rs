use std::path::Path;

#[test]
fn hoisted_mock_factories_reject_unsafe_shapes_and_accept_inert_values() {
    let source = r#"
        import { vi } from "vitest";
        const m1 = helper();
        const m2 = get().hoisted(() => ({ a: 1 }));
        const m3 = vi.hoisted(() => ({ a: 1 }), 1);
        const m4 = vi.hoisted(factory);
        const m5 = vi.hoisted(() => {
            const x = 1;
            return { x };
        });
        const m6 = vi.hoisted(() => ({ ...other }));
        const m7 = vi.hoisted(() => ({ nested: { a: 1 }, tpl: `ok`, fn: vi.fn() }));
        const m8 = vi.hoisted(() => {
            return { a: 1 };
        });
        vi.mock("./m1", () => m1);
        vi.mock("./m2", () => m2);
        vi.mock("./m3", () => m3);
        vi.mock("./m4", () => m4);
        vi.mock("./m5", () => m5);
        vi.mock("./m6", () => m6);
        vi.mock("./m7", () => m7);
        vi.mock("./m8", () => m8);
    "#;
    let facts = crate::ast::with_program(Path::new("hoisted.test.ts"), source, |program, _| {
        crate::module_facts::extract_with_arguments(program, false)
    })
    .unwrap();
    let results: Vec<_> = facts
        .module_mock_calls
        .iter()
        .map(|call| (call.specifier.as_deref(), call.full))
        .collect();
    assert_eq!(
        results,
        vec![
            (Some("./m1"), false),
            (Some("./m2"), false),
            (Some("./m3"), false),
            (Some("./m4"), false),
            (Some("./m5"), false),
            (Some("./m6"), false),
            (Some("./m7"), true),
            (Some("./m8"), true),
        ]
    );
}

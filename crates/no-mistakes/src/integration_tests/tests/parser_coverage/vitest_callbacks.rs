use super::*;

#[derive(serde::Deserialize)]
struct Shape {
    name: String,
    source: String,
    recovered: bool,
}

#[test]
fn static_callback_shapes_are_recovered_without_guessing_opaque_values() {
    let shapes: Vec<Shape> = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/integration-tests/vitest-callbacks/fixture/shapes.json"
    )))
    .unwrap();
    let root = fixture("vitest-callbacks");
    for shape in shapes {
        let path = root.join(format!("{}.config.mts", shape.name));
        let projects = parse_vitest_fixture(&shape.source, &path, &root).unwrap();
        assert_eq!(
            !projects.is_empty(),
            shape.recovered,
            "{}: {projects:?}",
            shape.name
        );
    }
}

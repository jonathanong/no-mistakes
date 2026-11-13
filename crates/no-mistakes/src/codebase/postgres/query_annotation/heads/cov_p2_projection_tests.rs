#[test]
fn cov_p2_callback_and_namespace_projections_keep_exact_runtime_shapes() {
    let (mut files, root) = super::prepared();
    let paths = files
        .keys()
        .cloned()
        .collect::<std::collections::BTreeSet<_>>();
    super::super::super::import_projection::project(&mut files, |specifier, from| {
        let path = from.parent()?.join(specifier).with_extension("mts");
        paths.get(&path).cloned()
    });
    for (name, count) in [
        ("callback-matrix", 10),
        ("namespace-matrix", 5),
        ("returned-matrix", 9),
    ] {
        let path = root.join(format!("{name}.mts"));
        let source = std::fs::read_to_string(&path).unwrap();
        let heads = files[&path].query_annotation_heads.as_ref().unwrap();
        let mut offset = 0;
        let mut checked = 0;
        for line in source.split_inclusive('\n') {
            if let Some((statement, expected)) = line.split_once("// low: ") {
                let start = offset + statement.find("read(").unwrap();
                let span = files[&path]
                    .embedded_sql
                    .iter()
                    .flat_map(|(_, embedded)| &embedded.call_spans)
                    .find(|span| span.0 as usize == start)
                    .unwrap_or_else(|| panic!("missing executor in {name}: {line}"));
                assert_eq!(
                    heads.accepts(*span),
                    expected.trim() == "true",
                    "{name}: {line}"
                );
                checked += 1;
            }
            offset += line.len();
        }
        assert_eq!(checked, count, "{name}");
    }
}

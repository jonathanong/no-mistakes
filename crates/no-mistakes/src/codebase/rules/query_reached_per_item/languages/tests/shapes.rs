use super::project_ext;

#[test]
fn loop_headers_boundaries_and_call_shapes_keep_per_item_calls() {
    let cases: &[(&str, &str, usize, &[&str])] = &[
        (
            "fn load(items: &[&str]) {\n    for item in items { example.store.read_row(item); }\n}\n",
            "rs",
            1,
            &["load → read_row"],
        ),
        (
            "fn load(items: &[&str]) {\n    while example.store.read_row(item) {\n    }\n}\n",
            "rs",
            1,
            &["while `example.store.read_row`", "load → read_row"],
        ),
        (
            "fn load(items: &[&str]) {\n    for item in example.store.read_row(items) {\n        item.fetch();\n    }\n}\n",
            "rs",
            1,
            &["item.fetch"],
        ),
        (
            "fn load(items: &[&str]) {\n    for item in items {\n        fn later() {\n            example.store.read_row(item);\n        }\n    }\n}\n",
            "rs",
            0,
            &[],
        ),
        (
            "fn fetch(item: &str) {\n    for row in item {\n        example.store.read_row(row);\n    }\n}\nfn load(items: &[&str]) {\n    for item in items {\n        fetch(item);\n    }\n}\n",
            "rs",
            2,
            &["for `example.store.read_row`", "for `fetch`"],
        ),
        (
            "fn load(items: &[&str]) {\n    for item in items {\n        if ready(item) {\n            example.store.read_row(item);\n        }\n    }\n}\n",
            "rs",
            2,
            &["load → read_row"],
        ),
        (
            "fn load(items: &[&str]) {\n    for item in items {\n        example.store.read_row (item);\n    }\n}\n",
            "rs",
            1,
            &["load → read_row"],
        ),
        (
            "fn load(items: &[&str]) {\n    for item in items {\n        example.store.read_row\n            (item);\n    }\n}\n",
            "rs",
            1,
            &["load → read_row"],
        ),
        (
            "void load(string[] items)\n{\n    foreach (var item in items)\n    {\n        example.store.read_row(item);\n    }\n}\n",
            "cs",
            1,
            &["foreach `example.store.read_row`", "load → read_row"],
        ),
        (
            "void load(string[] items)\n{\n    while (example.store.read_row(item))\n    {\n    }\n}\n",
            "cs",
            1,
            &["while `example.store.read_row`", "load → read_row"],
        ),
        (
            "def load(items):\n    for item in items:\n        label = \"keep // inside\"\n        note = \"\"\"\n        // example.store.read_row(other)\n        \"\"\"\n        escaped = \"a \\\" // still\"\n        kept = width // example.store.read_row(item)\n        tail = \"unterminated\n",
            "py",
            1,
            &["for `example.store.read_row`"],
        ),
        (
            "def load(items):\n    for item in items:\n        values = (\nitem\n        )\n        example.store.read_row(item)\n        note = )\n",
            "py",
            1,
            &["for `example.store.read_row`"],
        ),
        (
            "async def load(items):\n    async for item in items:\n        example.store.read_row(item)\n",
            "py",
            1,
            &["for `example.store.read_row`"],
        ),
        ("async def load(items):\n    async with items:\n        example.store.read_row(item)\n", "py", 0, &[]),
        (
            "def load(items)\n  items.each {\n    item.fetch()\n  }\n  example.store.read_row(item)\nend\n",
            "rb",
            1,
            &["each `item.fetch`"],
        ),
        (
            "def load(items)\n  items.each { item.fetch() }\n  example.store.read_row(item)\nend\n",
            "rb",
            1,
            &["each `item.fetch`"],
        ),
        (
            "def load(items)\n  items.each do\n    hash = { a: 1 }\n    example.store.read_row(item)\n  end\nend\n",
            "rb",
            1,
            &["each `example.store.read_row`"],
        ),
        (
            "def load(items)\n  stray = }\n  for item in items do\n    example.store.read_row(item)\n  end\nend\n",
            "rb",
            1,
            &["for `example.store.read_row`"],
        ),
    ];
    for (source, extension, len, needles) in cases {
        let findings = project_ext(source, extension);
        assert_eq!(
            findings.len(),
            *len,
            "{extension} {needles:?}\n{findings:#?}"
        );
        for needle in *needles {
            assert!(
                findings
                    .iter()
                    .any(|finding| finding.message.contains(needle)),
                "{needle}\n{findings:#?}"
            );
        }
        assert!(findings
            .iter()
            .all(|finding| !finding.message.contains("ready →")));
    }
}

#[test]
fn split_lines_inline_bodies_unbraced_loops_and_false_proofs() {
    let split = project_ext(
        "fn load(items: &[&str]) {\n    for item in items {\n        example.store.read_row\n            (item);\n    }\n}\n",
        "rs",
    );
    assert!(split
        .iter()
        .any(|finding| finding.line == 3 && finding.message.contains("read_row")));
    let java = project_ext(
        "void load(List<Row> items) {\n    for (Row row : example.store.read_row(items)) {\n        row.fetch();\n    }\n}\n",
        "java",
    );
    assert!(java
        .iter()
        .all(|finding| !finding.message.contains("read_row")));
    assert!(java
        .iter()
        .any(|finding| finding.message.contains("row.fetch")));
    let inline = project_ext(
        "def fetch(item): example.store.read_row(item)\ndef load(items):\n    for item in items:\n        fetch(item)\n",
        "py",
    );
    assert!(inline
        .iter()
        .any(|finding| finding.message.contains("load → fetch → read_row")));
    let unbraced = project_ext(
        "void load(string[] items)\n{\n    foreach (var item in items)\n        example.store.read_row(item);\n}\n",
        "cs",
    );
    assert!(unbraced
        .iter()
        .any(|finding| finding.message.contains("load → read_row")));
    let shadow = project_ext(
        "def fetch(item):\n    example.store.read_row(item)\ndef load(items, fetch):\n    for item in items:\n        fetch(item)\n",
        "py",
    );
    assert!(shadow.iter().any(|finding| {
        finding
            .causes
            .first()
            .is_some_and(|cause| cause.kind == "unresolved-dispatch")
            && finding.message.contains("`fetch`")
    }));
    assert!(shadow
        .iter()
        .all(|finding| !finding.message.contains("→ fetch →")));
    let lifetime = project_ext(
        "fn load(items: &[&str]) {\n    let value: &'static str = \"x\";\n    for row in items {\n        example.store.read_row(row);\n    }\n}\n",
        "rs",
    );
    assert!(lifetime
        .iter()
        .any(|finding| finding.message.contains("load → read_row")));
    let character = project_ext(
        "fn load(items: &[&str]) {\n    let ch = 'x';\n    for item in items {\n        example.store.read_row(item);\n    }\n}\n",
        "rs",
    );
    assert!(character
        .iter()
        .any(|finding| finding.message.contains("load → read_row")));
    let generic = project_ext(
        "void load(string[] items) {\n    foreach (var item in items) {\n        example.store.read_row<Row>(item);\n    }\n}\n",
        "cs",
    );
    assert!(generic
        .iter()
        .any(|finding| finding.message.contains("load → read_row")));
    let ruby = project_ext(
        "def load(items)\n  for item in items do\n    example.store.read_row item\n  end\nend\n",
        "rb",
    );
    assert!(ruby
        .iter()
        .any(|finding| finding.message.contains("load → read_row")));
}

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

#[test]
fn once_only_headers_keep_later_calls_in_the_loop() {
    let initializer = project_ext(
        "void load(List<Row> items) {\n    for (int i = example.store.read_row(items); i < count; i++) {\n        item.fetch();\n    }\n}\n",
        "java",
    );
    assert!(
        initializer
            .iter()
            .all(|finding| !finding.message.contains("read_row")),
        "{initializer:#?}"
    );
    let condition = project_ext(
        "void load(List<Row> items) {\n    for (int i = 0; example.store.read_row(items); i++) {\n    }\n}\n",
        "java",
    );
    assert!(
        condition
            .iter()
            .any(|finding| finding.message.contains("read_row")),
        "{condition:#?}"
    );
    let outer = project_ext(
        "void load(List<Row> items) {\n    for (Row item : items) {\n        for (int i = example.store.read_row(item); i < count; i++) {\n        }\n    }\n}\n",
        "java",
    );
    assert!(
        outer
            .iter()
            .any(|finding| finding.message.contains("read_row")),
        "{outer:#?}"
    );
    let keyword = project_ext(
        "fn load(items: &[&str]) {\n    for item in items {\n        other.mod.new(item);\n    }\n}\n",
        "rs",
    );
    assert!(
        keyword.iter().any(|finding| {
            finding
                .causes
                .first()
                .is_some_and(|cause| cause.kind == "unresolved-dispatch")
                && finding.message.contains("other.mod.new")
        }),
        "{keyword:#?}"
    );
    let carried = project_ext(
        "fn load(items: &[&str]) {\n    for item in items {\n        audit(); example.store.read_row\n            (item);\n    }\n}\n",
        "rs",
    );
    assert!(
        carried.iter().any(|finding| {
            finding.line == 3 && finding.message.contains("example.store.read_row")
        }),
        "{carried:#?}"
    );
    let ruby = project_ext(
        "def load(items)\n  for item in items do\n    example.store.read_row\n  end\nend\n",
        "rb",
    );
    assert!(
        ruby.iter()
            .any(|finding| finding.message.contains("load → read_row")),
        "{ruby:#?}"
    );
    let added = project_ext(
        "def load(items)\n  for item in items do\n    total = example.store.read_row + 1\n  end\nend\n",
        "rb",
    );
    assert!(
        added
            .iter()
            .all(|finding| !finding.message.contains("read_row")),
        "{added:#?}"
    );
    let elixir = project_ext(
        "defmodule Worker do\n  def load(items) do\n    for item <- items do\n      example.store.read_row\n    end\n  end\nend\n",
        "ex",
    );
    assert!(
        elixir
            .iter()
            .any(|finding| finding.message.contains("load → read_row")),
        "{elixir:#?}"
    );
    let unclosed = project_ext(
        "def helper(item):\n    example.store.read_row(item)\ndef run(\n    helper,\n    items,\n):\n    for item in items:\n        helper(item)\n",
        "py",
    );
    assert!(
        unclosed.iter().any(|finding| {
            finding
                .causes
                .first()
                .is_some_and(|cause| cause.kind == "unresolved-dispatch")
                && finding.message.contains("`helper`")
        }),
        "{unclosed:#?}"
    );
    assert!(
        unclosed
            .iter()
            .all(|finding| !finding.message.contains("→ helper →")),
        "{unclosed:#?}"
    );
    let raw = project_ext(
        "func load(items []string) {\n    url := `https://example.test`\n    for _, item := range items {\n        example.store.read_row(item)\n    }\n}\n",
        "go",
    );
    assert!(
        raw.iter()
            .any(|finding| finding.message.contains("load → read_row")),
        "{raw:#?}"
    );
    let unbraced = project_ext(
        "void load(List<Row> rows) {\n    for (Row row : rows)\n        if (ready(row))\n            example.store.read_row(row);\n}\n",
        "java",
    );
    assert!(
        unbraced
            .iter()
            .any(|finding| finding.message.contains("load → read_row")),
        "{unbraced:#?}"
    );
    let otherwise = project_ext(
        "void load(List<Row> rows) {\n    for (Row row : rows)\n        if (ready(row))\n            skip(row);\n        else\n            example.store.read_row(row);\n}\n",
        "java",
    );
    assert!(
        otherwise
            .iter()
            .any(|finding| finding.message.contains("load → read_row")),
        "{otherwise:#?}"
    );
    let assigned = project_ext(
        "fun fetch(item: Item) {\n    example.store.read_row(item)\n}\nfun load(items: List<Item>) {\n    for (item in items) {\n        val row = fetch(item)\n    }\n}\n",
        "kt",
    );
    assert!(
        assigned
            .iter()
            .any(|finding| finding.message.contains("load → fetch → read_row")),
        "{assigned:#?}"
    );
    let iterable = project_ext(
        "def load(items):\n    for item in (\n        example.store.read_row(items)\n    ):\n        pass\n",
        "py",
    );
    assert!(
        iterable
            .iter()
            .all(|finding| !finding.message.contains("read_row")),
        "{iterable:#?}"
    );
    let body = project_ext(
        "def load(items):\n    for item in (\n        items\n    ):\n        example.store.read_row(item)\n",
        "py",
    );
    assert!(
        body.iter()
            .any(|finding| finding.message.contains("load → read_row")),
        "{body:#?}"
    );
    let enclosed = project_ext(
        "def load(items):\n    for outer in items:\n        for item in (\n            example.store.read_row(outer)\n        ):\n            pass\n",
        "py",
    );
    assert!(
        enclosed
            .iter()
            .any(|finding| finding.message.contains("read_row")),
        "{enclosed:#?}"
    );
    let condition_lines = project_ext(
        "def load(items):\n    while (\n        example.store.read_row(items)\n    ):\n        pass\n",
        "py",
    );
    assert!(
        condition_lines
            .iter()
            .any(|finding| finding.message.contains("read_row")),
        "{condition_lines:#?}"
    );
}

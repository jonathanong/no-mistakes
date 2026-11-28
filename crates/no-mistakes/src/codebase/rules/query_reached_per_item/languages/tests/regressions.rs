use super::project_ext;

#[test]
fn inline_loops_trailing_statements_and_nested_iterables_keep_scopes() {
    let trailing = project_ext(
        "fn load(items: &[&str]) {\n    for item in items { audit(item); } example::store::read_row();\n}\n",
        "rs",
    );
    assert_eq!(trailing.len(), 1, "{trailing:#?}");
    assert!(trailing[0].message.contains("`audit`"), "{trailing:#?}");

    let nested_trailing = project_ext(
        "fn load(batches: &[&str]) {\n    for batch in batches {\n        for item in items { audit(item); } example::store::read_row(batch);\n    }\n}\n",
        "rs",
    );
    assert!(
        nested_trailing
            .iter()
            .any(|finding| finding.causes.is_empty() && finding.message.contains("read_row")),
        "{nested_trailing:#?}"
    );

    let inline_py = project_ext(
        "def load(items):\n    for item in items: example.store.read_row(item)\n    example.store.read_row(items)\n",
        "py",
    );
    assert_eq!(inline_py.len(), 1, "{inline_py:#?}");
    assert_eq!(inline_py[0].line, 2, "{inline_py:#?}");
    assert!(
        inline_py[0].message.contains("load → read_row"),
        "{inline_py:#?}"
    );

    let sliced_py = project_ext(
        "def load(items):\n    for item in items[1:2]: example.store.read_row(item)\n",
        "py",
    );
    assert_eq!(sliced_py.len(), 1, "{sliced_py:#?}");
    assert!(
        sliced_py[0].message.contains("load → read_row"),
        "{sliced_py:#?}"
    );

    let nested_iterable = project_ext(
        "def load(batches):\n    for batch in batches:\n        for item in example.store.read_row(batch):\n            pass\n",
        "py",
    );
    assert_eq!(nested_iterable.len(), 1, "{nested_iterable:#?}");
    assert!(
        nested_iterable[0].message.contains("load → read_row"),
        "{nested_iterable:#?}"
    );

    let pending_nested_iterable = project_ext(
        "void load(List<Row> batches) {\n    for (Row batch : batches) {\n        for (Row item : example.store.read_row(batch))\n        {\n        }\n    }\n}\n",
        "java",
    );
    assert_eq!(pending_nested_iterable.len(), 1, "{pending_nested_iterable:#?}");
    assert!(
        pending_nested_iterable[0]
            .message
            .contains("load → read_row"),
        "{pending_nested_iterable:#?}"
    );
}

#[test]
fn python_continuations_and_comments_preserve_statement_boundaries() {
    let separate = project_ext(
        "def load(items):\n    for item in items:\n        handler = example.store.read_row\n        (key, value) = pair\n",
        "py",
    );
    assert!(separate.is_empty(), "{separate:#?}");

    let backslash = project_ext(
        "def load(items):\n    for item in items:\n        example.store.read_row \\\n            (item)\n",
        "py",
    );
    assert_eq!(backslash.len(), 1, "{backslash:#?}");
    assert_eq!(backslash[0].line, 3, "{backslash:#?}");
    assert!(
        backslash[0].message.contains("load → read_row"),
        "{backslash:#?}"
    );

    let parens = project_ext(
        "def load(items):\n    for item in items:\n        result = (\n            example.store.read_row\n            (item)\n        )\n",
        "py",
    );
    assert_eq!(parens.len(), 1, "{parens:#?}");
    assert_eq!(parens[0].line, 4, "{parens:#?}");

    let comment_quote = project_ext(
        "# don't round here\ndef load(items):\n    for item in items:\n        value = item // example.store.read_row(item)\n",
        "py",
    );
    assert_eq!(comment_quote.len(), 1, "{comment_quote:#?}");
    assert!(
        comment_quote[0].message.contains("load → read_row"),
        "{comment_quote:#?}"
    );
}

#[test]
fn go_lexical_context_and_ruby_member_names_keep_frames_intact() {
    let go_string = project_ext(
        "func load(items []string) {\n    hint := \"`\"\n    rune := '`'\n    // backtick ` in line comment\n    /* backtick ` in block comment */\n    escaped := \"\\\"`\"\n    for _, item := range items {\n        example.store.read_row(item)\n    }\n}\n",
        "go",
    );
    assert_eq!(go_string.len(), 1, "{go_string:#?}");
    assert!(
        go_string[0].message.contains("load → read_row"),
        "{go_string:#?}"
    );

    let ruby_members = project_ext(
        "class Worker\n  def load(items)\n    for item in items do\n      kind = item.class\n      last = range.end\n      tag = { class: 1, end: 2 }\n      example.store.read_row(item)\n    end\n    example.store.read_row(other)\n  end\nend\n",
        "rb",
    );
    let proven: Vec<_> = ruby_members
        .iter()
        .filter(|finding| finding.causes.is_empty())
        .collect();
    assert_eq!(proven.len(), 1, "{ruby_members:#?}");
    assert_eq!(proven[0].line, 7, "{ruby_members:#?}");
    assert!(
        proven[0].message.contains("load → read_row"),
        "{ruby_members:#?}"
    );
    assert!(
        ruby_members.iter().all(|finding| finding.line != 9),
        "{ruby_members:#?}"
    );
}

#[test]
fn multiline_signatures_and_mixed_helper_bodies_prove_sinks() {
    let multiline = project_ext(
        "fn decl(\n    item: Item,\n);\nfn fetch(\n    item: Item,\n) -> Result<(), Error> {\n    example::store::read_row(item);\n}\nfn load(\n    items: &[Item],\n)\n{\n    for item in items {\n        fetch(item);\n    }\n}\n",
        "rs",
    );
    assert_eq!(multiline.len(), 1, "{multiline:#?}");
    assert!(multiline[0].causes.is_empty(), "{multiline:#?}");
    assert!(
        multiline[0].message.contains("load → fetch → read_row"),
        "{multiline:#?}"
    );

    let inline_tail = project_ext(
        "fn fetch(\n    item: Item,\n) { example::store::read_row(item); }\nfn load(items: &[Item]) {\n    for item in items {\n        fetch(item);\n    }\n}\n",
        "rs",
    );
    assert_eq!(inline_tail.len(), 1, "{inline_tail:#?}");
    assert!(
        inline_tail[0].message.contains("load → fetch → read_row"),
        "{inline_tail:#?}"
    );

    let reversed_mixed = project_ext(
        "fn fetch(item: &str) {\n    item.audit();\n    example.store.read_row(item);\n}\nfn load(items: &[&str]) {\n    for item in items {\n        fetch(item);\n    }\n}\n",
        "rs",
    );
    assert_eq!(reversed_mixed.len(), 1, "{reversed_mixed:#?}");
    assert!(reversed_mixed[0].causes.is_empty(), "{reversed_mixed:#?}");
    assert!(
        reversed_mixed[0].message.contains("load → fetch → read_row"),
        "{reversed_mixed:#?}"
    );

    let nested_shadow = project_ext(
        "fn outer(fetch: fn(&str)) {\n    fn inner(items: &[&str]) {\n        for item in items {\n            fetch(item);\n        }\n    }\n}\n",
        "rs",
    );
    assert_eq!(nested_shadow.len(), 1, "{nested_shadow:#?}");
    assert!(!nested_shadow[0].causes.is_empty(), "{nested_shadow:#?}");

    let unbraced_block = project_ext(
        "void load(List<Row> rows) {\n    for (Row row : rows)\n        if (ready(row)) {\n            example.store.read_row(row);\n        }\n    example.store.read_row(other);\n}\n",
        "java",
    );
    assert!(
        unbraced_block
            .iter()
            .any(|finding| finding.line == 4 && finding.message.contains("load → read_row")),
        "{unbraced_block:#?}"
    );
    assert!(
        unbraced_block.iter().all(|finding| finding.line != 6),
        "{unbraced_block:#?}"
    );
}

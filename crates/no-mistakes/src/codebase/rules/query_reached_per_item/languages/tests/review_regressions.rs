use super::project_ext;

#[test]
fn keywords_match_and_paren_free_floats_avoid_false_callees() {
    let php = project_ext(
        "function load($items) {\n    foreach ($items as $item) {\n        if (first($item)) {\n        } elseif (ready($item)) {\n        }\n    }\n}\n",
        "php",
    );
    assert!(
        php.iter().all(|f| f.import.as_deref() != Some("elseif")),
        "{php:#?}"
    );

    let py_not = project_ext(
        "def load(items):\n    for item in items:\n        if not (ready(item)):\n            pass\n",
        "py",
    );
    assert!(
        py_not.iter().all(|f| f.import.as_deref() != Some("not")),
        "{py_not:#?}"
    );

    let cs_using = project_ext(
        "void Load(string[] items) {\n    foreach (var item in items) {\n        using (var tx = open(item)) {\n        }\n    }\n}\n",
        "cs",
    );
    assert!(
        cs_using
            .iter()
            .all(|f| f.import.as_deref() != Some("using")),
        "{cs_using:#?}"
    );

    let rs_match = project_ext(
        "fn fetch(item: &str) {\n    example::store::read_row(item);\n}\nfn load(items: &[&str]) {\n    for item in items {\n        match (item, 1) {\n            _ => {}\n        }\n        match fetch(item) {\n            _ => {}\n        }\n    }\n}\n",
        "rs",
    );
    assert_eq!(rs_match.len(), 1, "{rs_match:#?}");
    assert!(
        rs_match[0].message.contains("load → fetch → read_row"),
        "{rs_match:#?}"
    );

    let py_match = project_ext(
        "def match(item):\n    example.store.read_row(item)\ndef load(items):\n    for item in items:\n        match(item)\n",
        "py",
    );
    assert_eq!(py_match.len(), 1, "{py_match:#?}");
    assert!(
        py_match[0].message.contains("load → match → read_row"),
        "{py_match:#?}"
    );

    let rb_float = project_ext(
        "def load(items)\n  for item in items do\n    sleep 0.5\n    rate = 1.5\n    3.times\n  end\nend\n",
        "rb",
    );
    assert_eq!(rb_float.len(), 1, "{rb_float:#?}");
    assert_eq!(
        rb_float[0].import.as_deref(),
        Some("3.times"),
        "{rb_float:#?}"
    );
}

#[test]
fn csharp_verbatim_same_line_loops_and_generic_headers_stay_accurate() {
    let verbatim = project_ext(
        "void Load(string[] items) {\n    var root = @\"C:\\\";\n    var interp = $@\"a\"\"b\nline2\\\";\n    var alt = @$\"D:\\\";\n    var norm = \"@\\\"\";\n    // @\"\n    /* @\" */\n    foreach (var item in items) {\n        example.store.read_row(item);\n    }\n}\n",
        "cs",
    );
    assert_eq!(verbatim.len(), 1, "{verbatim:#?}");
    assert_eq!(verbatim[0].line, 10, "{verbatim:#?}");
    assert!(
        verbatim[0].message.contains("Load → read_row"),
        "{verbatim:#?}"
    );

    let java_same = project_ext(
        "void load(List<Row> rows) {\n    for (Row r : fetchRows()) example.store.read_row(r);\n    Other.Call();\n}\n",
        "java",
    );
    assert_eq!(java_same.len(), 1, "{java_same:#?}");
    assert_eq!(java_same[0].line, 2, "{java_same:#?}");
    assert!(
        java_same[0].message.contains("load → read_row"),
        "{java_same:#?}"
    );

    let cs_same = project_ext(
        "void Load(Row[] rows) {\n    foreach (var r in fetchRows()) example.store.read_row(r);\n    Other.Call();\n}\n",
        "cs",
    );
    assert_eq!(cs_same.len(), 1, "{cs_same:#?}");
    assert!(
        cs_same[0].message.contains("Load → read_row"),
        "{cs_same:#?}"
    );

    let c_style_same = project_ext(
        "void load(int n) {\n    for (int i = example.store.read_row(0); i < n; i++) example.store.read_row(i);\n    Other.Call();\n}\n",
        "java",
    );
    assert_eq!(c_style_same.len(), 1, "{c_style_same:#?}");
    assert_eq!(c_style_same[0].line, 2, "{c_style_same:#?}");

    let braced_enhanced = project_ext(
        "void load(List<Row> rows) {\n    for (Row r : rows) { example.store.read_row(r); } Other.Call();\n}\n",
        "java",
    );
    assert_eq!(braced_enhanced.len(), 1, "{braced_enhanced:#?}");
    assert!(
        braced_enhanced[0].message.contains("load → read_row"),
        "{braced_enhanced:#?}"
    );

    let empty_semi = project_ext(
        "void load(List<Row> rows) {\n    for (Row r : rows);\n    Other.Call();\n}\n",
        "java",
    );
    assert!(empty_semi.is_empty(), "{empty_semi:#?}");

    let tuple_destructure = project_ext(
        "fn load(items: &[(i32, i32)]) {\n    for (a, b) in fetch(items) {\n        example::store::read_row(a);\n    }\n}\n",
        "rs",
    );
    assert_eq!(tuple_destructure.len(), 1, "{tuple_destructure:#?}");
    assert!(
        tuple_destructure[0].message.contains("load → read_row"),
        "{tuple_destructure:#?}"
    );

    let kotlin = project_ext(
        "fun <T> fetch(item: T) {\n    example.store.read_row(item)\n}\nfun <T> run(items: List<T>) {\n    for (item in items) {\n        fetch(item)\n    }\n}\n",
        "kt",
    );
    assert_eq!(kotlin.len(), 1, "{kotlin:#?}");
    assert!(
        kotlin[0].message.contains("run → fetch → read_row"),
        "{kotlin:#?}"
    );

    let csharp = project_ext(
        "void Process<T>(T item) {\n    example.store.read_row(item);\n}\nvoid Load<T>(T[] items) {\n    foreach (var item in items) {\n        Process(item);\n    }\n}\n",
        "cs",
    );
    assert_eq!(csharp.len(), 1, "{csharp:#?}");
    assert!(
        csharp[0].message.contains("Load → Process → read_row"),
        "{csharp:#?}"
    );

    let go = project_ext(
        "func Fetch[T any](item T) {\n    example.store.read_row(item)\n}\nfunc Map[T []int](xs T) {\n    for _, x := range xs {\n        Fetch(x)\n    }\n}\n",
        "go",
    );
    assert_eq!(go.len(), 1, "{go:#?}");
    assert!(go[0].message.contains("Map → Fetch → read_row"), "{go:#?}");
}

#[test]
fn ruby_until_hash_braces_and_unreadable_file_allowance_stay_accurate() {
    let until = project_ext(
        "def load(items)\n  items.each do |item|\n    until ready?(item)\n      wait(item)\n    end\n    example.store.read_row(item)\n  end\n  example.store.read_row(other)\nend\n",
        "rb",
    );
    assert!(
        until
            .iter()
            .any(|f| f.line == 4 && f.message.contains("until `wait`")),
        "{until:#?}"
    );
    assert!(
        until
            .iter()
            .any(|f| f.line == 6 && f.message.contains("load → read_row")),
        "{until:#?}"
    );
    assert!(until.iter().all(|f| f.line != 8), "{until:#?}");

    let postfix = project_ext(
        "def load(items)\n  items.each do |item|\n    x += 1 while cond\n    y += 1 until done\n  end\n  example.store.read_row(other)\nend\n",
        "rb",
    );
    assert!(postfix.is_empty(), "{postfix:#?}");

    let elixir_cond = project_ext(
        "defmodule Worker do\n  def load(items) do\n    for item <- items do\n      cond do\n        true -> example.store.read_row(item)\n      end\n    end\n    example.store.read_row(other)\n  end\nend\n",
        "ex",
    );
    assert_eq!(elixir_cond.len(), 1, "{elixir_cond:#?}");
    assert_eq!(elixir_cond[0].line, 5, "{elixir_cond:#?}");

    let hash_brace = project_ext(
        "def load(items)\n  items.each { |item|\n    options = {\n      key: item\n    }\n    example.store.read_row(item)\n  }\n  example.store.read_row(other)\nend\n",
        "rb",
    );
    assert_eq!(hash_brace.len(), 1, "{hash_brace:#?}");
    assert_eq!(hash_brace[0].line, 6, "{hash_brace:#?}");
    assert!(
        hash_brace[0].message.contains("load → read_row"),
        "{hash_brace:#?}"
    );

    let prefix_hash = project_ext(
        "def load(items)\n  foo({ a: 1 }).each { |item|\n    example.store.read_row(item)\n  }\n  example.store.read_row(other)\nend\n",
        "rb",
    );
    assert_eq!(prefix_hash.len(), 1, "{prefix_hash:#?}");
    assert_eq!(prefix_hash[0].line, 3, "{prefix_hash:#?}");

    let sinks = super::sinks();
    let file_allow = [super::super::super::config::Allowance {
        file: "worker.py".to_string(),
        function: None,
        callee: None,
        line: None,
    }];
    let usage = crate::codebase::rules::allow_audit::AllowUsage::new(1);
    let mut allowed = super::ctx(&sinks, crate::config::v2::Unanalyzable::Report);
    allowed.allow = &file_allow;
    allowed.usage = Some(&usage);
    assert!(super::project("worker.py", None, "py", &allowed).is_empty());
    assert_eq!(usage.unused().count(), 0);

    let narrow_allow = [super::super::super::config::Allowance {
        file: "worker.py".to_string(),
        function: Some("load".to_string()),
        callee: None,
        line: None,
    }];
    let mut narrow = super::ctx(&sinks, crate::config::v2::Unanalyzable::Report);
    narrow.allow = &narrow_allow;
    assert_eq!(super::project("worker.py", None, "py", &narrow).len(), 1);
}

use super::*;

#[test]
fn config_set_extractors_cover_edge_cases() {
    assert!(extract_yaml_sequence(":", "packages").is_empty());
    assert!(extract_yaml_sequence("packages: nope", "packages").is_empty());
    assert!(extract_yaml_sequence("updates: []", "updates.0.exclude").is_empty());
    assert_eq!(
        extract_yaml_sequence("packages:\n  - one\n  - 2\n", "packages"),
        BTreeSet::from(["one".to_string()])
    );

    assert!(extract_ts_array_literal("const OTHER = ['a'];", "NAMES").is_empty());
    assert_eq!(
        extract_ts_array_literal(
            r#"const NAMES = ["a\"b", `template`, notString, 'single'];"#,
            "NAMES"
        ),
        BTreeSet::from([
            "a\"b".to_string(),
            "single".to_string(),
            "template".to_string()
        ])
    );
    assert!(extract_ts_array_literal(
        r#"const NAMES = getNames(); const OTHER = ["api"];"#,
        "NAMES"
    )
    .is_empty());
    assert_eq!(
        extract_ts_array_literal(
            r#"const NAMES = [
  // keep pinned
  "@acme/api",
  /*
   * keep local
   */
  "@acme/web",
];"#,
            "NAMES"
        ),
        BTreeSet::from(["@acme/api".to_string(), "@acme/web".to_string()])
    );
    assert!(extract_ts_array_literal(r#"const NAMES = ["unterminated];"#, "NAMES").is_empty());
    assert!(
        extract_ts_const_array_property("const OTHER = [{ name: 'api' }];", "ITEMS", "name")
            .is_empty()
    );
    assert!(extract_ts_const_array_property(
        r#"const ITEMS = [{ name: "unterminated }];"#,
        "ITEMS",
        "name"
    )
    .is_empty());
    assert!(extract_ts_const_array_property(
        r#"const ITEMS = getItems(); const OTHER = [{ name: "api" }];"#,
        "ITEMS",
        "name"
    )
    .is_empty());
    assert_eq!(
        extract_ts_const_array_property(
            r#"const ITEMS = [
  // first-party package
  { name: "api" },
  "ignored",
  /* documented in policy table */
  { name: `web`, other: "x" },
];"#,
            "ITEMS",
            "name"
        ),
        BTreeSet::from(["api".to_string(), "web".to_string()])
    );
    assert!(extract_yaml_sequence("packages:\n  nested: []\n", "packages.0").is_empty());
    assert!(extract_sql_enum("CREATE TYPE status AS ENUM ('open'", "status").is_empty());
}

#[test]
fn comparison_modes_cover_defaults_custom_messages_and_unknown_modes() {
    let left = ExtractedSet {
        file: "left.ts".to_string(),
        values: BTreeSet::from(["api".to_string(), "web".to_string()]),
        issues: Vec::new(),
    };
    let right = ExtractedSet {
        file: "right.md".to_string(),
        values: BTreeSet::from(["api".to_string()]),
        issues: Vec::new(),
    };

    let mut findings = Vec::new();
    super::super::comparison::compare(
        &left,
        &right,
        &Comparison {
            left: "left".to_string(),
            right: "right".to_string(),
            message: Some("sets differ".to_string()),
            ..Default::default()
        },
        &mut findings,
    );
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].message, "sets differ");

    findings.clear();
    super::super::comparison::compare(
        &left,
        &right,
        &Comparison {
            left: "left".to_string(),
            right: "right".to_string(),
            mode: "unknown".to_string(),
            ..Default::default()
        },
        &mut findings,
    );
    assert!(findings.is_empty());
}

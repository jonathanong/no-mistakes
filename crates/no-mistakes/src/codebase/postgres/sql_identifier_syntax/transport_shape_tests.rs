use super::*;
use std::collections::BTreeSet;

#[test]
fn low_transport_shapes_preserve_unknowns_and_ignore_unrelated_writes() {
    let root =
        crate::test_support::rule_fixture_root("postgres-no-offset").join("low-transport-shapes");
    let mut expected = BTreeSet::new();
    for name in [
        "callbacks.mts",
        "mutations.ts",
        "parameters.mts",
        "aliases.mts",
        "tags.mts",
    ] {
        let file = format!("src/{name}");
        let source = std::fs::read_to_string(root.join(&file)).unwrap();
        expected.extend(source.lines().enumerate().filter_map(|(index, line)| {
            if line.contains("// ADVISE_NEXT") {
                Some((file.clone(), index + 2))
            } else {
                line.contains("// ADVISE")
                    .then_some((file.clone(), index + 1))
            }
        }));
    }
    let options = crate::napi_api::options::test_json_arg(serde_json::json!({
        "root": root, "config": root.join(".no-mistakes-low.yml")
    }));
    let report: serde_json::Value =
        serde_json::from_str(&crate::napi_api::check_json_impl(options).unwrap()).unwrap();
    assert!(report["rules"].as_array().unwrap().is_empty(), "{report}");
    assert!(
        report["warnings"].as_array().unwrap().is_empty(),
        "{report}"
    );
    for rule in [
        "postgres-no-offset",
        "postgres-no-unbounded-sql-identifier",
        "postgres-no-mutable-sql-registry",
    ] {
        let actual = report["advisories"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|row| row["rule"] == rule)
            .map(|row| {
                (
                    row["file"].as_str().unwrap().to_string(),
                    row["line"].as_u64().unwrap() as usize,
                )
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(actual, expected, "{rule}: {report}");
    }
}

#[test]
fn incomplete_alias_bindings_retain_the_original_executor_escape() {
    super::fixture("executor-alias.ts", |program, source, mut ts| {
        // A partial supplied binding table must not consume an untracked alias.
        std::sync::Arc::make_mut(ts.module_bindings.as_mut().unwrap())
            .bindings
            .retain(|binding| binding.name != "run");
        let profile = EmbeddedSqlOptions::configured("@example/db", &[]);
        let result = collect(program, &ts, source, &[profile]);
        assert_eq!(result.profiles[0].1.len(), 1);
        assert_eq!(result.profiles[0].1[0].line, 3);
        assert!(matches!(result.profiles[0].1[0].input, Input::Unknown));
    });
}

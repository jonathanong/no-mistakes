use super::{
    budget::Limits,
    catalog::PreparedCatalog,
    patch_support_tests as support,
    value::Value,
    values::{Cache, Engine},
};

#[test]
fn cov_p2_value_forms_keep_only_authored_sql_when_semantic_budgets_exhaust() {
    let files = support::prepare(&["cov-p2-value-budgets.mts"]);
    let path = super::tests::fixture().join("src/cov-p2-value-budgets.mts");
    let resolve = |name: &str, from: &std::path::Path| support::resolve(&files, from, name);
    let catalog = PreparedCatalog {
        files: &files,
        application: None,
        profile: None,
        resolve: &resolve,
    };
    for (name, expected) in [
        ("staticField", &["SELECT 1"][..]),
        ("computedField", &["SELECT 1"][..]),
        ("appendField", &["SELECT 1 /* suffix */"][..]),
        ("appendStatement", &["SELECT 1 /* suffix */"][..]),
        ("trueExpression", &["SELECT 1"][..]),
        ("falseExpression", &["SELECT 1"][..]),
        ("uncertainExpression", &["SELECT 1", "SELECT 2"][..]),
        ("trueStatement", &["SELECT 1"][..]),
        ("falseStatement", &["SELECT 1"][..]),
        ("uncertainStatement", &["SELECT 1", "SELECT 2"][..]),
        ("helperCall", &["SELECT 1"][..]),
        ("closedScalarCall", &["SELECT $1"][..]),
        ("textConcatenation", &["SELECT 1"][..]),
        ("numericComparison", &["SELECT 1", "SELECT 2"][..]),
        ("defaultMember", &["SELECT 1"][..]),
        ("defaultComputed", &["SELECT 1"][..]),
        ("interpolatedText", &["SELECT 1"][..]),
        ("optionalComputed", &["SELECT 1"][..]),
        ("aggregateArray", &["SELECT $1"][..]),
        ("logicalAnd", &["SELECT 1"][..]),
        ("logicalOr", &["SELECT 1"][..]),
        ("promiseDataMember", &["SELECT $1"][..]),
    ] {
        let key = catalog
            .binding(&path, support::binding(&files, &path, name))
            .unwrap();
        let args = if name.starts_with("uncertain") || name == "numericComparison" {
            vec![Value::Scalar]
        } else {
            vec![]
        };
        let mut exhausted = false;
        let mut complete = false;
        for work in 0..80 {
            for variants in [1, 16] {
                let mut cache = Cache::default();
                let mut engine = Engine::new(
                    &catalog,
                    &mut cache,
                    Limits {
                        work,
                        variants,
                        ..Limits::default()
                    },
                );
                let values = engine.run(&key, &args).unwrap_or_default();
                exhausted |= values.is_empty();
                let mut sql = values
                    .iter()
                    .map(|value| {
                        let (Value::Sql(text) | Value::Text(text)) = value else {
                            panic!("{name}: expected SQL or text")
                        };
                        assert!(text.valid_binds(), "{name}");
                        assert!(
                            expected.contains(&text.raw.sql.as_str()),
                            "{name}: {}",
                            text.raw.sql
                        );
                        text.raw.sql.as_str()
                    })
                    .collect::<Vec<_>>();
                sql.sort_unstable();
                if !engine.residual() && sql == expected {
                    complete = true;
                }
            }
        }
        assert!(
            exhausted && complete,
            "{name}: must fail closed and recover with sufficient budget"
        );
    }
}

#[test]
fn cov_p2_promised_sql_and_unbound_template_values_never_become_scalar_exemptions() {
    let files = support::prepare(&["cov-p2-value-budgets.mts"]);
    let path = super::tests::fixture().join("src/cov-p2-value-budgets.mts");
    let catalog = PreparedCatalog {
        files: &files,
        application: None,
        profile: None,
        resolve: &|_, _| None,
    };
    for name in [
        "promiseSqlMember",
        "promiseAsData",
        "expressionWithoutBindings",
    ] {
        let key = catalog
            .binding(&path, support::binding(&files, &path, name))
            .unwrap();
        let mut cache = Cache::default();
        let mut engine = Engine::new(&catalog, &mut cache, Limits::default());
        let values = engine.run(&key, &[]).unwrap();
        assert!(values.is_empty(), "{name} must not recover SQL");
        assert!(engine.residual(), "{name} must retain uncertainty");
    }
}

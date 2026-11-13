use super::{
    budget::Limits,
    catalog::PreparedCatalog,
    patch_support_tests as support,
    value::Value,
    values::{Cache, Engine},
};

#[test]
fn runtime_helper_imports_require_real_callable_exports_through_reexports() {
    let files = support::prepare(&[
        "real-catalog-query.mts",
        "real-catalog-helpers.mts",
        "real-catalog-alias.mts",
        "real-catalog-types.mts",
        "real-catalog-namespace.mts",
    ]);
    let caller = super::tests::fixture().join("src/real-catalog-query.mts");
    let source = std::fs::read_to_string(&caller).unwrap();
    let resolve =
        |specifier: &str, from: &std::path::Path| support::resolve(&files, from, specifier);
    let catalog = PreparedCatalog {
        files: &files,
        application: None,
        profile: None,
        resolve: &resolve,
    };
    let calls = &files[&caller].sql_identifier_syntax.recovery.calls;
    let mut checked = 0;
    let mut mismatches = Vec::new();
    for call in calls {
        let raw = &source[call.span.0 as usize..call.span.1 as usize];
        if !raw.starts_with("read(") {
            continue;
        }
        let mut cache = Cache::default();
        let mut engine = Engine::new(&catalog, &mut cache, Limits::default());
        let values = engine.evaluate_at(&caller, &call.arguments[0], None);
        let expected = raw == "read(make())" || raw == "read(renamed())";
        if expected {
            let valid = values.as_ref().is_some_and(|values| values.len() == 1 && matches!(&values[0], Value::Sql(text) if text.raw.sql == "SELECT 1" && text.valid_binds()));
            if !valid || engine.residual() {
                mismatches.push(format!("{raw}: expected one complete SQL result"));
            }
        } else if values.is_some() || !engine.residual() {
            mismatches.push(format!(
                "{raw}: non-callable, missing, or erased exports must remain uncertain"
            ));
        }
        checked += 1;
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
    assert_eq!(checked, 8);
}

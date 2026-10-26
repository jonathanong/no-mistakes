use super::{
    budget::Limits,
    catalog::PreparedCatalog,
    patch_support_tests as support,
    value::Value,
    values::{Cache, Engine},
};

#[test]
fn prepared_catalog_follows_exports_and_preserves_mutation_cycle_and_failure_uncertainty() {
    let names = [
        "patch-catalog.mts",
        "patch-catalog-helper.mts",
        "patch-reexport.mts",
        "patch-cycle-a.mts",
        "patch-cycle-b.mts",
    ];
    let mut files = support::prepare(&names);
    let caller = super::tests::fixture().join("src/patch-catalog.mts");
    let source = std::fs::read_to_string(&caller).unwrap();
    {
        let resolve =
            |specifier: &str, from: &std::path::Path| support::resolve(&files, from, specifier);
        let catalog = PreparedCatalog {
            files: &files,
            application: None,
            profile: None,
            resolve: &resolve,
        };
        let mut checked = 0;
        for call in &files[&caller].sql_identifier_syntax.recovery.calls {
            let raw = &source[call.span.0 as usize..call.span.1 as usize];
            if !raw.starts_with("read(") {
                continue;
            }
            let mut cache = Cache::default();
            let mut engine = Engine::new(&catalog, &mut cache, Limits::default());
            let values = engine.evaluate(&caller, &call.arguments[0]);
            if raw.contains("forwarded") {
                let values = values.unwrap();
                assert_eq!(values.len(), 1);
                assert!(matches!(&values[0], Value::Text(text) if text.raw.sql == "SELECT 1"));
                assert!(!engine.residual());
            } else {
                assert!(values.is_none(), "{raw}");
                assert!(engine.residual(), "{raw}");
            }
            checked += 1;
        }
        assert_eq!(checked, 3);
    }
    // Prepared I/O/parse failure must invalidate the helper proof, not invent SQL.
    let helper = super::tests::fixture().join("src/patch-catalog-helper.mts");
    files.get_mut(&helper).unwrap().parse_error = Some("prepared source failure".into());
    let resolve =
        |specifier: &str, from: &std::path::Path| support::resolve(&files, from, specifier);
    let catalog = PreparedCatalog {
        files: &files,
        application: None,
        profile: None,
        resolve: &resolve,
    };
    let call = files[&caller]
        .sql_identifier_syntax
        .recovery
        .calls
        .iter()
        .find(|call| {
            source[call.span.0 as usize..call.span.1 as usize].starts_with("read(forwarded")
        })
        .unwrap();
    let mut cache = Cache::default();
    let mut engine = Engine::new(&catalog, &mut cache, Limits::default());
    assert!(engine.evaluate(&caller, &call.arguments[0]).is_none());
    assert!(engine.residual());
}

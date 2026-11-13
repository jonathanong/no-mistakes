use super::{
    catalog::PreparedCatalog,
    patch_support_tests as support,
    values::{Catalog, Key},
};
use crate::codebase::{dependencies::extract::CallableId, postgres::EmbeddedSqlOptions};

#[test]
fn cov_p2_prepared_catalog_refuses_missing_files_and_mismatched_executor_profiles() {
    let files = support::prepare(&["cov-p2-value-budgets.mts"]);
    let path = super::tests::fixture().join("src/cov-p2-value-budgets.mts");
    let missing = path.with_file_name("cov-p2-not-prepared.mts");
    let profile = super::tests::profile();
    let other = EmbeddedSqlOptions::configured("@other/db", &[]);
    {
        let catalog = PreparedCatalog {
            files: &files,
            application: None,
            profile: Some(&profile),
            resolve: &|_, _| None,
        };
        let key = catalog
            .binding(&path, support::binding(&files, &path, "staticField"))
            .unwrap();
        assert!(catalog.function(&key).is_some());
        assert!(catalog
            .typed_parameters(
                &path,
                support::binding(&files, &path, "staticField"),
                &profile
            )
            .is_some());
        assert!(catalog
            .typed_parameters(
                &path,
                support::binding(&files, &path, "staticField"),
                &other
            )
            .is_none());
        assert!(catalog.binding(&missing, 0).is_none());
        assert!(catalog
            .function(&Key {
                file: missing.clone(),
                callable: CallableId(0)
            })
            .is_none());
        assert!(!catalog.sink(&missing, 0));
    }
}

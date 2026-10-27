use crate::codebase::rules::run_check;

fn findings() -> Vec<crate::codebase::rules::RuleFinding> {
    run_check(&super::tests::fixture("workspace-exports"), None, None).unwrap()
}

#[test]
fn workspace_exports_map_imports_stay_concrete_and_test_only_exports_report() {
    let found = findings();
    assert!(
        found.iter().any(|row| {
            row.file == "app/src/api.mts"
                && row.line == 5
                && row.target.as_deref() == Some("testOnly")
        }),
        "{found:?}"
    );
    assert!(
        found
            .iter()
            .all(|row| row.file != "app/src/consumer.mts"
                && row.file != "packages/pkg/src/index.mts"),
        "{found:?}"
    );
    assert!(
        found.iter().all(|row| {
            !row.message
                .contains("unresolved module selector `@scope/pkg`")
                && !row
                    .message
                    .contains("unresolved module selector `@scope/pkg/feature`")
        }),
        "{found:?}"
    );
}

#[test]
fn unresolved_specifiers_name_the_specifier_and_import_line() {
    let found = findings();
    let absent = found
        .iter()
        .find(|row| {
            row.file == "app/src/unresolved.mts"
                && row.message.contains("unresolved symbol `absent`")
        })
        .unwrap_or_else(|| panic!("{found:?}"));
    assert_eq!(absent.line, 3, "{absent:?}");
    assert!(absent.message.contains("from `@scope/pkg`"), "{absent:?}");
    let missing = found
        .iter()
        .find(|row| {
            row.message
                .contains("unresolved module selector `@scope/pkg/missing`")
        })
        .unwrap_or_else(|| panic!("{found:?}"));
    assert_eq!(missing.file, "app/src/unresolved.mts");
    assert_eq!(missing.line, 4, "{missing:?}");
    let gone = found
        .iter()
        .find(|row| {
            row.message
                .contains("unresolved module selector `./gone.mjs`")
        })
        .unwrap_or_else(|| panic!("{found:?}"));
    assert_eq!(gone.file, "app/src/unresolved.mts");
    assert_eq!(gone.line, 5, "{gone:?}");
}

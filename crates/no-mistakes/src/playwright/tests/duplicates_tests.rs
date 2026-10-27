use crate::playwright::analysis::duplicates::build_duplicate_selectors;
use crate::playwright::analysis::types::UniqueSelectorPolicy;
use crate::playwright::selectors::{self, AppSelectorValue};
use std::path::{Path, PathBuf};

#[test]
fn duplicate_selector_report_includes_exact_values_only() {
    let root = Path::new("/repo");
    let app_selectors = vec![
        selectors::AppSelector {
            file: PathBuf::from("/repo/web/app/b.tsx"),
            attribute: "data-pw".to_string(),
            value: AppSelectorValue::Exact("same".to_string()),
            line: 1,
            recovered: false,
        },
        selectors::AppSelector {
            file: PathBuf::from("/repo/web/app/b.tsx"),
            attribute: "data-pw".to_string(),
            value: AppSelectorValue::Exact("same".to_string()),
            line: 1,
            recovered: false,
        },
        selectors::AppSelector {
            file: PathBuf::from("/repo/web/app/b.tsx"),
            attribute: "data-testid".to_string(),
            value: AppSelectorValue::Exact("same".to_string()),
            line: 1,
            recovered: false,
        },
        selectors::AppSelector {
            file: PathBuf::from("/repo/web/app/a.tsx"),
            attribute: "data-testid".to_string(),
            value: AppSelectorValue::Exact("same".to_string()),
            line: 1,
            recovered: false,
        },
        selectors::AppSelector {
            file: PathBuf::from("/repo/web/app/c.tsx"),
            attribute: "data-testid".to_string(),
            value: AppSelectorValue::Unsupported("id".to_string()),
            line: 1,
            recovered: false,
        },
        selectors::AppSelector {
            file: PathBuf::from("/repo/web/app/d.tsx"),
            attribute: "data-testid".to_string(),
            value: AppSelectorValue::Exact("unique".to_string()),
            line: 1,
            recovered: false,
        },
    ];

    let duplicates = build_duplicate_selectors(
        root,
        &app_selectors,
        UniqueSelectorPolicy {
            test_ids: true,
            html_ids: false,
            ..UniqueSelectorPolicy::default()
        },
    );
    assert_eq!(duplicates.len(), 4);
    assert_eq!(duplicates[0].file, "web/app/a.tsx");
    assert_eq!(duplicates[1].attribute, "data-pw");
    assert_eq!(duplicates[2].attribute, "data-pw");
    assert_eq!(duplicates[3].attribute, "data-testid");
}

#[test]
fn duplicate_selector_report_keeps_html_ids_separate_from_test_ids() {
    let root = Path::new("/repo");
    let app_selectors = vec![
        selectors::AppSelector {
            file: PathBuf::from("/repo/web/app/a.tsx"),
            attribute: "data-testid".to_string(),
            value: AppSelectorValue::Exact("same".to_string()),
            line: 1,
            recovered: false,
        },
        selectors::AppSelector {
            file: PathBuf::from("/repo/web/app/b.tsx"),
            attribute: "id".to_string(),
            value: AppSelectorValue::Exact("same".to_string()),
            line: 1,
            recovered: false,
        },
    ];

    let duplicates = build_duplicate_selectors(
        root,
        &app_selectors,
        UniqueSelectorPolicy {
            test_ids: true,
            html_ids: true,
            ..UniqueSelectorPolicy::default()
        },
    );
    assert!(duplicates.is_empty());
}

#[test]
fn deprecated_duplicate_selector_report_preserves_aggregate_grouping() {
    let root = Path::new("/repo");
    let app_selectors = vec![
        selectors::AppSelector {
            file: PathBuf::from("/repo/web/app/a.tsx"),
            attribute: "data-testid".to_string(),
            value: AppSelectorValue::Exact("same".to_string()),
            line: 1,
            recovered: false,
        },
        selectors::AppSelector {
            file: PathBuf::from("/repo/web/app/b.tsx"),
            attribute: "id".to_string(),
            value: AppSelectorValue::Exact("same".to_string()),
            line: 1,
            recovered: false,
        },
    ];

    let duplicates = build_duplicate_selectors(
        root,
        &app_selectors,
        UniqueSelectorPolicy {
            test_ids: true,
            html_ids: true,
            aggregate: true,
            ..UniqueSelectorPolicy::default()
        },
    );
    assert_eq!(duplicates.len(), 2);
}

#[test]
fn configured_html_id_selectors_count_as_test_ids_for_uniqueness() {
    let root = Path::new("/repo");
    let app_selectors = vec![
        selectors::AppSelector {
            file: PathBuf::from("/repo/web/app/a.tsx"),
            attribute: "id".to_string(),
            value: AppSelectorValue::Exact("same".to_string()),
            line: 1,
            recovered: false,
        },
        selectors::AppSelector {
            file: PathBuf::from("/repo/web/app/b.tsx"),
            attribute: "id".to_string(),
            value: AppSelectorValue::Exact("same".to_string()),
            line: 1,
            recovered: false,
        },
    ];

    let duplicates = build_duplicate_selectors(
        root,
        &app_selectors,
        UniqueSelectorPolicy {
            test_ids: true,
            configured_html_id_selector: true,
            ..UniqueSelectorPolicy::default()
        },
    );
    assert_eq!(duplicates.len(), 2);
}

#[test]
fn literal_test_ids_only_skips_recovered_exact_values() {
    let root = Path::new("/repo");
    let app_selectors = vec![
        selectors::AppSelector {
            file: PathBuf::from("/repo/web/app/a.tsx"),
            attribute: "data-testid".to_string(),
            value: AppSelectorValue::Exact("same".to_string()),
            line: 4,
            recovered: false,
        },
        selectors::AppSelector {
            file: PathBuf::from("/repo/web/app/b.tsx"),
            attribute: "data-testid".to_string(),
            value: AppSelectorValue::Exact("same".to_string()),
            line: 8,
            recovered: true,
        },
    ];

    let skipped = build_duplicate_selectors(
        root,
        &app_selectors,
        UniqueSelectorPolicy {
            test_ids: true,
            literal_test_ids_only: true,
            ..UniqueSelectorPolicy::default()
        },
    );
    assert!(skipped.is_empty());

    let recovered = build_duplicate_selectors(
        root,
        &app_selectors,
        UniqueSelectorPolicy {
            test_ids: true,
            ..UniqueSelectorPolicy::default()
        },
    );
    assert_eq!(recovered.len(), 2);
}

#[test]
fn html_id_and_test_id_literal_policies_are_independent() {
    let root = Path::new("/repo");
    let app_selectors: Vec<_> = ["id", "data-testid"]
        .into_iter()
        .flat_map(|attribute| {
            (0..3).map(move |index| selectors::AppSelector {
                file: PathBuf::from(format!("/repo/web/app/{attribute}-{index}.tsx")),
                attribute: attribute.to_string(),
                value: AppSelectorValue::Exact("same".to_string()),
                line: 1,
                recovered: index == 2,
            })
        })
        .collect();

    // A max consumer can retain recovered facts while the other rule is low.
    for (html_low, test_low, html_count, test_count) in [
        (false, false, 3, 3),
        (true, false, 2, 3),
        (false, true, 3, 2),
        (true, true, 2, 2),
    ] {
        let duplicates = build_duplicate_selectors(
            root,
            &app_selectors,
            UniqueSelectorPolicy {
                test_ids: true,
                html_ids: true,
                literal_test_ids_only: test_low,
                literal_html_ids_only: html_low,
                ..UniqueSelectorPolicy::default()
            },
        );
        for (attribute, expected) in [("id", html_count), ("data-testid", test_count)] {
            let actual: Vec<_> = duplicates
                .iter()
                .filter(|selector| selector.attribute == attribute)
                .collect();
            assert_eq!(actual.len(), expected, "{html_low} {test_low} {attribute}");
            assert_eq!(
                actual
                    .iter()
                    .any(|selector| selector.file.ends_with("-2.tsx")),
                expected == 3,
                "{html_low} {test_low} {attribute}",
            );
        }
    }
}

#[test]
fn duplicate_selectors_keep_each_occurrence_line() {
    // Two occurrences in one file must each report their own attribute line.
    let selector = |line| selectors::AppSelector {
        file: PathBuf::from("/repo/web/app/a.tsx"),
        attribute: "data-testid".to_string(),
        value: AppSelectorValue::Exact("same".to_string()),
        line,
        recovered: false,
    };
    let duplicates = build_duplicate_selectors(
        Path::new("/repo"),
        &[selector(9), selector(3)],
        UniqueSelectorPolicy {
            test_ids: true,
            ..UniqueSelectorPolicy::default()
        },
    );
    let lines: Vec<_> = duplicates.iter().map(|duplicate| duplicate.line).collect();
    assert_eq!(lines, vec![3, 9]);
}

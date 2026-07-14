use super::{cached_analysis, canonical_filter_key, same_config_path, CachedAnalysis};
use std::cell::Cell;
use std::collections::HashMap;
use std::path::Path;

#[test]
fn same_config_path_normalizes_relative_paths_and_preserves_optionality() {
    let root = Path::new("/repo");

    assert!(same_config_path(
        root,
        Some(Path::new("config/../no-mistakes.yml")),
        Some(Path::new("/repo/no-mistakes.yml")),
    ));
    assert!(same_config_path(root, None, None));
    assert!(!same_config_path(
        root,
        Some(Path::new("no-mistakes.yml")),
        None,
    ));
}

#[test]
fn filter_cache_keys_ignore_order_and_duplicates() {
    let left = vec![
        "src/**".to_string(),
        "tests/**".to_string(),
        "src/**".to_string(),
    ];
    let right = vec!["tests/**".to_string(), "src/**".to_string()];
    assert_eq!(
        canonical_filter_key(&left).unwrap(),
        canonical_filter_key(&right).unwrap()
    );
}

#[test]
fn report_caches_call_each_analyzer_once_per_canonical_key() {
    let key = canonical_filter_key(&[
        "src/**".to_string(),
        "tests/**".to_string(),
        "src/**".to_string(),
    ])
    .unwrap();
    let equivalent_key =
        canonical_filter_key(&["tests/**".to_string(), "src/**".to_string()]).unwrap();

    for domain in ["queue", "server"] {
        let plain_calls = Cell::new(0);
        let indexed_calls = Cell::new(0);
        let mut plain = HashMap::new();
        let mut indexed = HashMap::new();

        for traversal in [false, false, true, true] {
            let report = cached_analysis(
                &mut plain,
                &mut indexed,
                if traversal { &equivalent_key } else { &key },
                traversal,
                || {
                    plain_calls.set(plain_calls.get() + 1);
                    Ok(format!("{domain}-plain"))
                },
                || {
                    indexed_calls.set(indexed_calls.get() + 1);
                    Ok(format!("{domain}-indexed"))
                },
            )
            .unwrap();
            match (traversal, report) {
                (false, CachedAnalysis::Plain(report)) => {
                    assert_eq!(report, &format!("{domain}-plain"));
                }
                (true, CachedAnalysis::Indexed(report)) => {
                    assert_eq!(report, &format!("{domain}-indexed"));
                }
                _ => panic!("{domain} selected the wrong analyzer"),
            }
        }

        assert_eq!(plain_calls.get(), 1, "{domain} plain analyzer");
        assert_eq!(indexed_calls.get(), 1, "{domain} indexed analyzer");
    }
}

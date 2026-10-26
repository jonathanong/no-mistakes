use super::*;

#[test]
fn malformed_selection_uses_only_prepared_source_and_never_recovers_exports() {
    let root = normalize_path(&fixture("malformed-selected"));
    let broken = normalize_path(&root.join("components/Broken.tsx"));
    let disabled = normalize_path(&root.join("components/FileDisabled.tsx"));
    let mut shared = crate::codebase::check_facts::collect_check_facts(
        &root,
        vec![broken.clone(), disabled.clone()],
        CheckFactPlan {
            react: true,
            symbols: true,
            source: true,
            ..Default::default()
        },
    );
    let opts: Options = serde_yaml::from_str("include: ['components/*.tsx']").unwrap();
    let include = GlobMatcher::new(&opts.include).unwrap();
    let exclude = GlobMatcher::new(&opts.exclude).unwrap();
    let cfg = config("");
    let test_filter = crate::codebase::test_filter::TestFileFilter::new(&root, &cfg);
    let select = |facts: &CheckFactMap| {
        selection::selected_components(
            &root,
            &root,
            facts,
            selection::SelectionOptions {
                options: &opts,
                defer_suppression: false,
            },
            &include,
            &exclude,
            &test_filter,
        )
    };
    let facts = shared.ts.get(&broken).unwrap();
    assert!(facts.parse_error.is_some());
    assert!(facts.react.is_none());
    let (components, diagnostics) = select(&shared);
    assert!(components.is_empty());
    assert_eq!(diagnostics.len(), 1);

    // The disk fixture has no exemption. Prepared text must be authoritative.
    let source = shared.ts.get(&disabled).unwrap().source.clone();
    std::sync::Arc::get_mut(shared.ts.get_mut(&broken).unwrap())
        .unwrap()
        .source = source;
    assert!(select(&shared).1.is_empty());

    // A recorded read failure also has no source; explicit selection still applies.
    std::sync::Arc::get_mut(shared.ts.get_mut(&broken).unwrap())
        .unwrap()
        .source = None;
    for (error, expected) in [
        ("cannot read selected source", "cannot read selected source"),
        (
            "cannot read selected source\ninternal detail",
            "cannot read selected source",
        ),
        ("", ""),
    ] {
        std::sync::Arc::get_mut(shared.ts.get_mut(&broken).unwrap())
            .unwrap()
            .parse_error = Some(error.to_string());
        let diagnostics = select(&shared).1;
        assert_eq!(diagnostics.len(), 1);
        let detail = if expected.is_empty() {
            String::new()
        } else {
            format!(" ({expected})")
        };
        assert!(diagnostics[0]
            .message
            .contains(&format!("could not be analyzed{detail},")));
        assert!(!diagnostics[0].message.contains("internal detail"));
        assert!(!diagnostics[0].message.contains('\n'));
    }
    shared.files.retain(|file| file != &broken);
    assert!(select(&shared).1.is_empty());
}

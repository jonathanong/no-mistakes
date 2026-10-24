use super::*;

#[test]
fn namespace_constants_follow_prepared_exports_and_preserve_unknown_reads() {
    let files = run("namespace-const-sql");
    let calls = caller_calls(&files);
    assert_eq!(calls.len(), 23, "{calls:#?}");
    let owner = variant_root("namespace-const-sql").join("src/constants.mts");
    for (call, line) in calls[..7].iter().zip([2, 3, 10, 2, 2, 10, 2]) {
        assert!(!call.is_unanalyzable(), "{call:?}");
        assert_eq!(
            call.sql_text.as_deref(),
            Some("SELECT id FROM orders WHERE account_id = $1")
        );
        assert!(
            call.sql_source_positions.iter().any(|position| {
                position.source_path.as_deref() == Some(owner.as_path())
                    && position.source_line == line
            }),
            "the literal retains its declaring module: {call:?}"
        );
    }
    assert_eq!(
        calls[7].sql_text.as_deref(),
        Some("SELECT id FROM orders WHERE true AND account_id = $1")
    );
    assert_eq!(
        calls[8].sql_text.as_deref(),
        Some("SELECT id FROM orders WHERE true")
    );
    for call in &calls[9..20] {
        assert!(call.is_unanalyzable(), "{call:?}");
        assert!(
            call.sql_text
                .as_deref()
                .is_none_or(|sql| !sql.contains('\0')),
            "{call:?}"
        );
    }
    assert!(
        !calls[20].is_unanalyzable(),
        "the local object fallback survives: {:?}",
        calls[20]
    );
    assert_eq!(
        calls[20].sql_text.as_deref(),
        Some("SELECT id FROM orders WHERE account_id = $1")
    );
    assert_eq!(
        calls[21].sql_text.as_deref(),
        Some("SELECT id FROM orders WHERE account_id = $1")
    );
    assert!(!calls[21].is_unanalyzable());
    assert!(
        calls[22].is_unanalyzable(),
        "append of an untagged template must still stringify its fragment"
    );
}

#[test]
fn namespace_constants_do_not_resolve_through_a_named_import() {
    use crate::codebase::dependencies::extract::ImportedBindingKind;
    let mut files = prepared("namespace-const-sql");
    let caller = files
        .keys()
        .find(|path| path.ends_with("src/caller.mts"))
        .unwrap()
        .clone();
    let ts = Arc::make_mut(&mut files.get_mut(&caller).unwrap().ts);
    let import = ts
        .imported_bindings
        .iter_mut()
        .find(|binding| binding.local == "ns")
        .unwrap();
    import.kind = ImportedBindingKind::Named;
    import.imported = "TABLE".to_string();
    let calls = Mutex::new(BTreeMap::new());
    let resolve = resolver(&files, &calls);
    project(&mut files, resolve);
    assert!(caller_calls(&files)[0].is_unanalyzable());
}

#[test]
fn namespace_constant_projection_reuses_the_request_parse_pass() {
    use crate::codebase::check_facts::{collect_check_facts, CheckFactPlan};
    let saved = variant_root("namespace-const-sql");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = normalize_path(fixture.path());
    let paths = ["caller", "barrel", "constants"]
        .map(|name| normalize_path(&root.join(format!("src/{name}.mts"))));
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    crate::ast::begin_parse_count(&root);
    let facts = collect_check_facts(
        &root,
        paths.to_vec(),
        CheckFactPlan {
            embedded_sql: true,
            embedded_sql_options: vec![options.clone()],
            ..Default::default()
        },
    );
    let counts = crate::ast::finish_parse_count(&root);
    for path in &paths {
        assert_eq!(counts.get(path), Some(&1), "{counts:#?}");
    }
    let calls = &facts.embedded_sql(&paths[0], &options).unwrap().calls;
    assert_eq!(
        calls[0].sql_text.as_deref(),
        Some("SELECT id FROM orders WHERE account_id = $1")
    );
    assert!(!calls[0].is_unanalyzable(), "{calls:#?}");
}

#[test]
fn namespace_constant_fill_preserves_one_origin_per_sql_byte() {
    use super::super::program::Shared;
    let files = prepared("namespace-const-sql");
    let (caller, file) = files
        .iter()
        .find(|(path, _)| path.ends_with("src/caller.mts"))
        .unwrap();
    let (options, embedded) = &file.embedded_sql[0];
    let calls = Mutex::new(BTreeMap::new());
    let mut shared = Shared::new(&files, Locator::new(&files, resolver(&files, &calls)));
    let mut recovered = 0;
    for pending in &embedded.pending_imported {
        for (_, site) in &pending.sites {
            if site.member.is_none() {
                continue;
            }
            if let Some(texts) = shared.fill(0, options, caller, site) {
                for text in texts {
                    assert!(!text.sql.is_empty());
                    assert_eq!(text.origins.len(), text.sql.len(), "{}", text.sql);
                    recovered += 1;
                }
            }
        }
    }
    assert!(
        recovered >= 10,
        "successful expansions must exercise the origin invariant"
    );
}

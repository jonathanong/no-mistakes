use super::*;
use crate::codebase::postgres::TrustedSqlTag;

fn root() -> PathBuf {
    normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-closed-identifiers/projection"),
    )
}

fn prepared_hasown() -> FileIdMap<CheckFileFacts> {
    let mut paths = discover_visible_paths(&root().join("src"));
    paths.retain(|path| path.extension().is_some_and(|extension| extension == "mts"));
    paths.sort();
    let options =
        EmbeddedSqlOptions::configured("@example/db", &[]).with_raw_sql_tags(&[TrustedSqlTag {
            module: "@example/raw-sql".into(),
            name: "sql".into(),
        }]);
    paths
        .into_iter()
        .map(|path| {
            let source = std::fs::read_to_string(&path).expect("fixture source");
            let file = crate::ast::with_program(&path, &source, |program, _| CheckFileFacts {
                ts: Arc::new(collect_file_facts_from_program(
                    &path,
                    TsFactPlan {
                        imports: true,
                        module_bindings: true,
                        ..Default::default()
                    },
                    &TsFactContext::default(),
                    &source,
                    program,
                    None,
                    None,
                )),
                embedded_sql: vec![(
                    options.clone(),
                    extract_embedded_sql_from_program(&path, program, &source, &options),
                )],
                ..Default::default()
            })
            .expect("parse");
            (path, file)
        })
        .collect()
}

fn sqls(calls: &[crate::codebase::postgres::EmbeddedSqlCall], marker: &str) -> Vec<String> {
    let call = calls
        .iter()
        .find(|call| {
            call.sql_text
                .as_deref()
                .is_some_and(|sql| sql.contains(marker))
                || call
                    .variants
                    .iter()
                    .any(|variant| variant.sql_text.contains(marker))
        })
        .unwrap_or_else(|| panic!("missing {marker}: {calls:#?}"));
    if call.variants.is_empty() {
        call.sql_text.iter().cloned().collect()
    } else {
        call.variants
            .iter()
            .map(|variant| variant.sql_text.clone())
            .collect()
    }
}

#[test]
fn imported_hasown_records_fill_string_holes_and_zip_nested_fields() {
    let mut files = prepared_hasown();
    let resolutions = Mutex::new(BTreeMap::new());
    let resolve = resolver(&files, &resolutions);
    project(&mut files, resolve);
    let calls = caller_calls(&files);
    assert_eq!(
        sqls(&calls, "/* both */"),
        [
            "/* both */ DELETE FROM users WHERE id = $1",
            "/* both */ DELETE FROM posts WHERE id = $1",
        ]
    );
    assert_eq!(
        sqls(&calls, "/* one */"),
        ["/* one */ DELETE FROM users WHERE id = $1"]
    );
    assert_eq!(
        sqls(&calls, "/* corr */"),
        [
            "/* corr */ DELETE FROM users WHERE user_id = $1",
            "/* corr */ DELETE FROM posts WHERE post_id = $1",
        ]
    );
    assert_eq!(
        sqls(&calls, "/* raw */"),
        [
            "/* raw */ DELETE FROM users WHERE id = $1",
            "/* raw */ DELETE FROM posts WHERE id = $1",
        ]
    );
    let loose = calls
        .iter()
        .find(|call| {
            call.sql_text
                .as_deref()
                .is_some_and(|sql| sql.contains("/* loose */"))
                || call
                    .variants
                    .iter()
                    .any(|variant| variant.sql_text.contains("/* loose */"))
        })
        .unwrap_or_else(|| panic!("missing loose call: {calls:#?}"));
    assert!(loose.is_unanalyzable(), "{loose:#?}");
}

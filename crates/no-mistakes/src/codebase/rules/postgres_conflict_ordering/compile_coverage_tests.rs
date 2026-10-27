use super::*;

#[test]
fn compile_options_reject_invalid_include_glob_and_unanalyzable_sql() {
    let glob_error = match compile_options(&Options {
        schema_catalog_path: "schema.json".to_string(),
        include: vec!["[".to_string()],
        ..Default::default()
    }) {
        Ok(_) => panic!("invalid include glob should fail"),
        Err(error) => error,
    };
    assert!(glob_error.to_string().contains("include"), "{glob_error:#}");

    let sql_error = match compile_options(&Options {
        schema_catalog_path: "schema.json".to_string(),
        unanalyzable_sql: "maybe".to_string(),
        ..Default::default()
    }) {
        Ok(_) => panic!("invalid unanalyzableSql should fail"),
        Err(error) => error,
    };
    assert!(
        sql_error.to_string().contains("unanalyzableSql"),
        "{sql_error:#}"
    );
}

#[test]
fn compile_options_cover_include_exclude_sql_sources_and_safe_directive() {
    let compiled = compile_options(&Options {
        schema_catalog_path: "schema.json".to_string(),
        include: vec!["src/**/*.ts".to_string()],
        exclude: vec!["src/skip.ts".to_string()],
        sql_include: vec!["queries/**/*.sql".to_string()],
        safe_directive: "SAFE".to_string(),
        unanalyzable_sql: "fail".to_string(),
        ..Default::default()
    })
    .unwrap();
    assert!(compiled.includes("src/insert.ts"));
    assert!(!compiled.includes("src/skip.ts"));
    assert!(!compiled.includes("lib/insert.ts"));
    assert!(compiled.excludes("src/skip.ts"));
    assert!(!compiled.excludes("src/insert.ts"));
    assert_eq!(compiled.safe_directive, "SAFE");
    assert!(compiled.sql_sources.is_some());
    assert!(compiled.fail_unanalyzable);
}

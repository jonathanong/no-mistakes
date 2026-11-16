//! Saved shapes whose check SQL must stay complete after helper appends.

use super::*;
use std::collections::BTreeMap;

fn case_root(rule: &str, name: &str) -> PathBuf {
    normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules")
            .join(rule)
            .join(name),
    )
}

fn prepared(root: &Path) -> FileIdMap<CheckFileFacts> {
    let mut paths = discover_visible_paths(&root.join("src"));
    paths.retain(|path| {
        path.extension()
            .is_some_and(|extension| extension == "ts" || extension == "mts")
    });
    paths.sort();
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
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

fn project_case(rule: &str, name: &str) -> FileIdMap<CheckFileFacts> {
    let mut files = prepared(&case_root(rule, name));
    let known = files.keys().cloned().collect::<FxHashSet<_>>();
    project(&mut files, move |specifier, from| {
        let base = from.parent()?.join(specifier);
        for candidate in [
            base.clone(),
            base.with_extension("ts"),
            base.with_extension("mts"),
            base.with_extension("tsx"),
            base.with_extension("js"),
        ] {
            let target = normalize_path(&candidate);
            if known.contains(&target) {
                return Some(target);
            }
        }
        None
    });
    files
}

fn sql_by_file(files: &FileIdMap<CheckFileFacts>) -> BTreeMap<String, Vec<String>> {
    let mut out = BTreeMap::new();
    for (path, file) in files {
        let mut texts = Vec::new();
        for call in &file.embedded_sql[0].1.calls {
            assert!(!call.is_unanalyzable(), "{path:?} {call:?}");
            if let Some(sql) = &call.sql_text {
                texts.push(sql.clone());
            }
            for variant in &call.variants {
                if !texts.contains(&variant.sql_text) {
                    texts.push(variant.sql_text.clone());
                }
            }
            assert!(
                call.sql_text.is_some() || !call.variants.is_empty(),
                "{path:?} line {}",
                call.line
            );
        }
        if !texts.is_empty() {
            out.insert(
                path.file_name().unwrap().to_string_lossy().into_owned(),
                texts,
            );
        }
    }
    out
}

#[test]
fn cross_module_append_keeps_for_update_parseable() {
    let sql = sql_by_file(&project_case(
        "postgres-lock-ordering",
        "cross-module-claim",
    ));
    let claim = &sql["claim.ts"];
    assert!(claim.iter().any(|sql| {
        sql.contains("failed_id IS NULL")
            && sql.contains("FOR UPDATE OF work")
            && sql.contains("sql_placeholder_5")
            && !sql.contains("30000")
            && !sql.contains("AND  ORDER")
    }));
    assert!(claim
        .iter()
        .any(|sql| sql == "SELECT id FROM work FOR UPDATE OF"));
}

#[test]
fn statement_append_keeps_both_arms() {
    let sql = sql_by_file(&project_case("postgres-lock-ordering", "statement-append"));
    let lock = &sql["lock.ts"];
    assert!(lock.iter().any(|sql| {
        sql.contains("placement.a_id = sql_placeholder_1")
            && sql.contains("FOR UPDATE OF placement")
    }));
    assert!(lock.iter().any(|sql| {
        sql.contains("placement.b_id = sql_placeholder_1")
            && sql.contains("FOR UPDATE OF placement")
    }));
    assert!(lock
        .iter()
        .any(|sql| sql == "SELECT id FROM placements FOR UPDATE OF"));
}

#[test]
fn plain_string_append_splices_raw_sql() {
    let sql = sql_by_file(&project_case(
        "postgres-required-predicates",
        "raw-column-append",
    ));
    let list = &sql["list.ts"];
    assert!(list.len() >= 2, "{list:?}");
    for text in list {
        assert!(text.contains("s.item_id = c.id"), "{text}");
        assert!(!text.contains("s.item_id = sql_placeholder"), "{text}");
        assert!(text.contains("AND s.visible"), "{text}");
    }
}

#[test]
fn imported_numeric_limit_stays_a_bind() {
    let files = project_case("postgres-conflict-ordering", "imported-limit");
    for (_, file) in &files {
        for call in &file.embedded_sql[0].1.calls {
            assert_eq!(call.kind, EmbeddedSqlKind::Inline, "{call:?}");
        }
    }
    let insert = &sql_by_file(&files)["insert.ts"];
    assert!(insert.len() >= 2, "{insert:?}");
    for text in insert {
        assert!(text.contains("LIMIT sql_placeholder_4"), "{text}");
        assert!(
            text.contains("ON CONFLICT (conversation_id, user_id)"),
            "{text}"
        );
        assert!(!text.contains("100"), "{text}");
        assert!(!text.contains("$1"), "{text}");
    }
}

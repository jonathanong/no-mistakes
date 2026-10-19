use super::{assert_inlined, assert_unknown, read_calls, EmbeddedSqlKind};

fn sqls(name: &str) -> Vec<String> {
    let calls = read_calls(name);
    assert!(!calls.is_empty(), "{name}");
    let mut sqls = Vec::new();
    for call in &calls {
        assert!(!call.is_unanalyzable(), "{name}: {call:?}");
        for version in call.statement_calls() {
            let sql = version.sql_text.clone().unwrap_or_default();
            assert!(!sql.contains('\0'), "{name}: {sql}");
            sqls.push(sql);
        }
    }
    sqls.sort();
    sqls
}

#[test]
fn member_ternary_keeps_both_alternatives() {
    let sqls = sqls("const-splice-member-ternary.ts");
    assert_eq!(
        sqls,
        vec![
            "SELECT id FROM orders".to_string(),
            "SELECT id FROM orders".to_string(),
            "SELECT id FROM orders OFFSET 10".to_string(),
            "SELECT id FROM orders OFFSET 10".to_string(),
        ]
    );
}

#[test]
fn member_conditional_append_keeps_both_alternatives() {
    let sqls = sqls("const-splice-member-conditional-append.ts");
    assert!(
        sqls.iter()
            .any(|sql| sql == "SELECT id FROM orders OFFSET 10"),
        "{sqls:?}"
    );
    assert!(
        sqls.iter().any(|sql| sql == "SELECT id FROM orders"),
        "{sqls:?}"
    );
    assert!(sqls.iter().all(|sql| !sql.contains('\0')), "{sqls:?}");
}

#[test]
fn poisoned_member_keeps_the_literal_offset() {
    let calls = read_calls("const-splice-member-write-literal-offset.ts");
    assert_eq!(calls.len(), 2, "{calls:?}");
    let table = calls[0].sql_text.as_deref().unwrap_or("");
    assert_eq!(table, "SELECT id FROM sql_placeholder_1 OFFSET 10");
    let numeric = calls[1].sql_text.as_deref().unwrap_or("");
    assert!(numeric.contains("OFFSET"), "{numeric}");
    assert!(numeric.contains("sql_placeholder_1"), "{numeric}");
    assert!(!numeric.contains('\0'), "{numeric}");
}

#[test]
fn tagged_const_piece_stays_unknown() {
    assert_unknown("const-splice-tagged-const-piece.ts");
    let calls = read_calls("const-splice-tagged-const-piece.ts");
    let sql = calls[0].sql_text.as_deref().unwrap_or("");
    assert!(!sql.contains("OFFSET"), "{sql}");
}

#[test]
fn exported_object_stays_unknown_and_exported_scalar_is_inlined() {
    assert_unknown("const-splice-exported-object.ts");
    assert_inlined("const-splice-exported-scalar.ts", EmbeddedSqlKind::Inline);
}

#[test]
fn hoisted_var_and_function_shadows_stay_unknown() {
    assert_unknown("const-splice-shadow-var-hoisted.ts");
    assert_unknown("const-splice-shadow-fn-decl.ts");
}

use super::{extract_sql_statement_facts, SqlValueForm};
use crate::codebase::postgres::parse::parse_postgres_sql;
use sqlparser::ast::Statement;

#[test]
fn assignment_forms_cover_named_wildcard_exists_and_in_subquery() {
    let named = extract_sql_statement_facts(
        "INSERT INTO items (id, note) VALUES (1, 'a')
         ON CONFLICT (id) DO UPDATE SET note = concat(a => EXCLUDED.note);",
    );
    assert!(
        matches!(
            named.inserts[0].on_conflict.as_ref().unwrap().assignments[0].form,
            SqlValueForm::Other | SqlValueForm::Excluded { .. }
        ) || !named.inserts[0]
            .on_conflict
            .as_ref()
            .unwrap()
            .assignments
            .is_empty()
    );
    let wildcard = extract_sql_statement_facts(
        "INSERT INTO items (id, n) VALUES (1, 0)
         ON CONFLICT (id) DO UPDATE SET n = count(*);",
    );
    assert_eq!(
        wildcard.inserts[0]
            .on_conflict
            .as_ref()
            .unwrap()
            .assignments[0]
            .form,
        SqlValueForm::Other
    );
    let exists = extract_sql_statement_facts(
        "INSERT INTO items (id, flag) VALUES (1, true)
         ON CONFLICT (id) DO UPDATE SET flag = EXISTS (SELECT 1);",
    );
    assert_eq!(
        exists.inserts[0].on_conflict.as_ref().unwrap().assignments[0].form,
        SqlValueForm::Subquery
    );
    let in_sub = extract_sql_statement_facts(
        "INSERT INTO items (id, flag) VALUES (1, true)
         ON CONFLICT (id) DO UPDATE SET flag = id IN (SELECT 1);",
    );
    assert_eq!(
        in_sub.inserts[0].on_conflict.as_ref().unwrap().assignments[0].form,
        SqlValueForm::Subquery
    );
    let dollar = extract_sql_statement_facts(
        "INSERT INTO items (id, note) VALUES (1, 'a')
         ON CONFLICT (id) DO UPDATE SET note = $1;",
    );
    assert_eq!(
        dollar.inserts[0].on_conflict.as_ref().unwrap().assignments[0].form,
        SqlValueForm::Placeholder
    );
}

#[test]
fn quote_comment_and_dollar_masks_skip_insert_keywords() {
    assert_eq!(
        extract_sql_statement_facts("SELECT 'it''s INSERT INTO x';").insert_keyword_count,
        0
    );
    assert_eq!(
        extract_sql_statement_facts("SELECT \"INSERT INTO x\";").insert_keyword_count,
        0
    );
    assert_eq!(
        extract_sql_statement_facts("SELECT E'it''s INSERT INTO x';").insert_keyword_count,
        0
    );
    assert_eq!(
        extract_sql_statement_facts("SELECT E'foo\\").insert_keyword_count,
        0
    );
    assert_eq!(
        extract_sql_statement_facts("SELECT 'unclosed INSERT INTO x").insert_keyword_count,
        0
    );
    let unclosed_comment = extract_sql_statement_facts("SELECT 1 /* INSERT INTO x");
    assert_eq!(unclosed_comment.insert_keyword_count, 0);
    assert_eq!(
        extract_sql_statement_facts("SELECT $tag$INSERT INTO decoy").insert_keyword_count,
        0
    );
    assert!(
        extract_sql_statement_facts("SELECT $1; INSERT INTO items (id) VALUES (1);")
            .insert_keyword_count
            >= 1
    );
    let multiline = extract_sql_statement_facts(
        "SELECT $tag$\nINSERT INTO decoy\n$tag$;\nINSERT INTO items (id) VALUES (1);",
    );
    assert_eq!(multiline.inserts.len(), 1);
    let quoted_nl = extract_sql_statement_facts(
        "SELECT '\nINSERT INTO decoy\n'; INSERT INTO items (id) VALUES (1);",
    );
    assert_eq!(quoted_nl.inserts.len(), 1);
    let comment_nl = extract_sql_statement_facts(
        "SELECT 1 /* INSERT\nINTO decoy */;\nINSERT INTO items (id) VALUES (1);",
    );
    assert_eq!(comment_nl.inserts.len(), 1);
}

#[test]
fn top_level_not_exists_respects_paren_depth_and_token_edges() {
    assert!(!super::has_top_level_not_exists_in(
        "SELECT 1 WHERE (NOT EXISTS (SELECT 1))"
    ));
    assert!(super::has_top_level_not_exists_in(
        "AND NOT EXISTS (SELECT 1)"
    ));
    assert!(super::has_top_level_not_exists_in(
        "INSERT INTO items (id) SELECT 1 WHERE\nNOT\tEXISTS (SELECT 1)"
    ));
    assert!(!super::has_top_level_not_exists_in("WHERE NOT EXISTSfoo"));
}

#[test]
fn nested_block_comments_do_not_hide_following_insert() {
    let facts =
        extract_sql_statement_facts("/* outer /* inner */ ' */ INSERT INTO items (id) VALUES (1);");
    assert!(facts.insert_keyword_count >= 1, "{facts:?}");
    assert_eq!(facts.inserts.len(), 1);
}

#[test]
fn doubled_quotes_and_unclosed_dollar_do_not_steal_insert_lines() {
    let doubled = extract_sql_statement_facts(
        "SELECT 'it''s INSERT INTO decoy';\nINSERT INTO items (id) VALUES (1);",
    );
    assert_eq!(doubled.inserts.len(), 1);
    assert_eq!(doubled.inserts[0].line, 2);
    let unclosed = extract_sql_statement_facts(
        "SELECT $tag$INSERT INTO decoy\nINSERT INTO items (id) VALUES (1);",
    );
    assert!(unclosed.inserts.is_empty() || unclosed.insert_keyword_count == 0);
    assert!(!super::has_top_level_not_exists_in("WHERE NOT"));
    assert!(!super::has_top_level_not_exists_in("AND"));
}

#[test]
fn unclosed_block_comment_does_not_drop_insert_assignments() {
    let sql = "INSERT INTO items (id, seen) VALUES (1, 'a')";
    let Statement::Insert(insert) = parse_postgres_sql(sql).unwrap().pop().unwrap() else {
        panic!("insert");
    };
    assert!(!super::insert::from_insert(
        "INSERT INTO items (id, seen) VALUES (1, 'a') /*",
        &insert,
        1,
        true,
    )
    .assignments
    .is_empty());
}

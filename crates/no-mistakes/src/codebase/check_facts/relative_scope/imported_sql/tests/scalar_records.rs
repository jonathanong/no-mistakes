use super::*;
use crate::codebase::postgres::EmbeddedSqlCall;

fn projected() -> FileIdMap<CheckFileFacts> {
    let mut files = prepared("scalar-record-members");
    let resolutions = Mutex::new(BTreeMap::new());
    let resolve = resolver(&files, &resolutions);
    project(&mut files, resolve);
    files
}

fn marked<'a>(calls: &'a [EmbeddedSqlCall], marker: &str) -> &'a EmbeddedSqlCall {
    calls
        .iter()
        .find(|call| {
            call.statement_calls().any(|statement| {
                statement
                    .sql_text
                    .as_deref()
                    .is_some_and(|sql| sql.contains(marker))
            })
        })
        .unwrap_or_else(|| panic!("missing {marker}: {calls:?}"))
}

#[test]
fn scalar_binds_keep_numbering_beside_frozen_fields_and_only_attribute_sql_helpers() {
    let files = projected();
    let calls = caller_calls(&files);
    let call = marked(&calls, "/* mixed */");
    assert!(!call.is_unanalyzable(), "{call:?}");
    let sql: Vec<_> = call
        .statement_calls()
        .map(|statement| statement.sql_text.clone().unwrap())
        .collect();
    assert_eq!(
        sql,
        ["/* mixed */ SELECT id, $1, $2, $3 FROM items WHERE id = $4"]
    );
    let (_, file) = files
        .iter()
        .find(|(path, _)| path.ends_with("src/caller.mts"))
        .unwrap();
    let embedded = &file.embedded_sql[0].1;
    let index = embedded
        .calls
        .iter()
        .position(|candidate| candidate.line == call.line)
        .unwrap();
    assert_eq!(
        embedded.imported_call_lines[&embedded.call_spans[index]],
        vec![call.line; 3]
    );
}

#[test]
fn ordinary_templates_and_relations_do_not_use_scalar_member_bind_fallback() {
    let files = projected();
    let calls = caller_calls(&files);
    assert_eq!(calls.len(), 3);
    for marker in ["/* ordinary */", "/* relation */"] {
        let call = marked(&calls, marker);
        assert!(call.is_unanalyzable(), "{marker}: {call:?}");
    }
}

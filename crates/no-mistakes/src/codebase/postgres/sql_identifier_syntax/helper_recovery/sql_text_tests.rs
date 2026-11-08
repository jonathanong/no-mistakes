use super::{coordinate, RecoveredSql, SqlText};
use crate::codebase::postgres::EmbeddedSqlSourcePosition;

fn fragment(name: &str, site: u32) -> SqlText {
    let sql = std::fs::read_to_string(
        crate::test_support::rule_fixture_root("postgres-site-identity").join(name),
    )
    .unwrap();
    let binds: Vec<_> = regex::Regex::new(r"\$\d+")
        .unwrap()
        .find_iter(&sql)
        .map(|bind| (bind.start(), bind.end()))
        .collect();
    let positions = binds
        .iter()
        .map(|&(start, _)| {
            let (sql_line, sql_column) = coordinate(&sql, start).unwrap();
            EmbeddedSqlSourcePosition {
                sql_line,
                sql_column,
                source_line: 1,
                source_path: None,
            }
        })
        .collect();
    let mut raw = RecoveredSql {
        origins: vec![site; sql.len()],
        sql,
        line: 1,
        positions,
        invocations: Default::default(),
    };
    raw.invocations.record(raw.sql.len(), site);
    SqlText {
        raw,
        binds,
        executor_payload: false,
    }
}

#[test]
fn appended_invocations_follow_generated_bind_width_changes() {
    let mut prefix = fragment("bind-prefix.sql", 11);
    prefix.append(fragment("bind-suffix.sql", 22)).unwrap();
    assert!(prefix.valid_binds());
    assert_eq!(prefix.raw.invocations.at(&prefix.raw.sql, (1, 1)), [11]);
    assert_eq!(prefix.raw.invocations.at(&prefix.raw.sql, (2, 1)), [22]);
    let end = prefix.raw.sql.lines().nth(1).unwrap().chars().count() + 1;
    assert_eq!(prefix.raw.invocations.at(&prefix.raw.sql, (2, end)), [22]);
    assert!(prefix
        .raw
        .invocations
        .at(&prefix.raw.sql, (3, 1))
        .is_empty());
}

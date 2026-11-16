use super::*;
use crate::codebase::postgres::embedded::TrustedSqlTag;
use crate::codebase::postgres::embedded::tests::{fixture, read_fixture};

fn programs() -> Programs {
    let path = fixture("programs/raw-sql.ts");
    let source = read_fixture("programs/raw-sql.ts");
    extract_embedded_sql_from_source(
        &path,
        &source,
        &EmbeddedSqlOptions::configured("@example/db", &[]).with_raw_sql_tags(&[TrustedSqlTag {
            module: "@example/raw-sql".into(),
            name: "sql".into(),
        }]),
    )
    .exported_programs
}

#[test]
fn configured_raw_tags_execute_inside_lowered_helpers() {
    let programs = programs();
    for name in ["rawStatement", "rawComposed"] {
        assert_eq!(
            exec(&programs, name, &[lit("users")]),
            Some(vec!["sql:SELECT id FROM users WHERE id = $1".into()]),
            "{name}"
        );
        assert_eq!(exec(&programs, name, &[SiteArg::Scalar]), None, "{name}");
    }
    assert_eq!(
        exec(&programs, "rawConstant", &[]),
        Some(vec!["sql:SELECT 1".into()])
    );
    assert_eq!(
        exec(&programs, "rawChoice", &[lit("drop")]),
        Some(vec!["sql:DROP TABLE users".into()])
    );
    assert_eq!(
        exec(&programs, "rawChoice", &[lit("read")]),
        Some(vec!["sql:SELECT id FROM users".into()])
    );
    let mut unknown = exec(&programs, "rawChoice", &[SiteArg::Scalar]).unwrap();
    unknown.sort();
    assert_eq!(
        unknown,
        ["sql:DROP TABLE users", "sql:SELECT id FROM users"]
    );
    assert_eq!(
        exec(&programs, "boundValue", &[SiteArg::Scalar]),
        Some(vec!["sql:SELECT id FROM users WHERE name = $1".into()])
    );
}

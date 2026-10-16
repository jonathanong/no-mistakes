use no_mistakes::codebase::postgres::{
    parse_postgres_source, PostgresSqlSource, PostgresSqlStatementKind,
};
use std::{fs, io::Write};

fn main() {
    let arguments: Vec<_> = std::env::args_os().skip(1).collect();
    assert_eq!(
        arguments.len(),
        3,
        "SQL, identity golden, and full-facts golden paths"
    );
    let sql = fs::read_to_string(&arguments[0]).expect("saved SQL fixture");
    let expected: Vec<String> =
        serde_json::from_slice(&fs::read(&arguments[1]).expect("saved legacy identity strings"))
            .expect("identity golden JSON");
    assert_eq!(expected.len(), 5);
    let facts = parse_postgres_source(&PostgresSqlSource {
        sql,
        file_name: None,
    });
    assert!(facts.diagnostics.is_empty(), "{:?}", facts.diagnostics);
    let identities: Vec<_> = facts
        .statements
        .iter()
        .map(|statement| {
            let PostgresSqlStatementKind::CreateIndex { index } = &statement.facts else {
                panic!("saved fixture must contain only indexes");
            };
            index.structural_identity.as_str()
        })
        .collect();
    assert_eq!(identities, expected);
    let encoded = serde_json::to_vec(&facts).expect("public SQL facts JSON");
    let legacy = fs::read(&arguments[2]).expect("retained default CJS/ESM facts golden");
    assert_eq!(
        encoded, legacy,
        "full public facts bytes must retain the legacy contract"
    );
    std::io::stdout()
        .lock()
        .write_all(&encoded)
        .expect("full raw output");
}

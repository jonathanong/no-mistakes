use super::proven;
use crate::codebase::check_facts::{CheckFactMap, CheckFileFacts};
use crate::codebase::postgres::EmbeddedSqlCall;
use crate::codebase::rules::postgres_sql_cross_file::SqlCall;
use crate::codebase::rules::postgres_sql_cross_file::{compile, Bindings, Options, Provenance};
use crate::codebase::ts_source::facts::TsFileFacts;
use std::path::PathBuf;
use std::sync::Arc;

#[test]
fn missing_facts_and_unmatched_calls_stay_unproven() {
    let opts = compile("postgres-no-unbounded-sql-identifier", &Options::default()).unwrap();
    let file = PathBuf::from("src/query.ts");
    let embedded = EmbeddedSqlCall::default();
    let call = SqlCall {
        file: file.clone(),
        span: (10, 20),
        call: &embedded,
    };
    let reference = Provenance::Reference {
        name: "table".into(),
        scope: 0,
        offset: 0,
        members: Vec::new(),
    };
    let facts = CheckFactMap::default();
    let bindings = Bindings::new(&facts);
    assert!(!proven(
        &opts,
        &facts,
        &bindings,
        &call,
        &Provenance::ProcessEnv("SCHEMA".into()),
    ));
    assert!(!proven(&opts, &facts, &bindings, &call, &reference));

    let mut facts = CheckFactMap::default();
    facts.ts.insert(
        file.clone(),
        CheckFileFacts {
            ts: Arc::new(TsFileFacts::default()),
            ..Default::default()
        }
        .into(),
    );
    let bindings = Bindings::new(&facts);
    assert!(!proven(&opts, &facts, &bindings, &call, &reference));
}

use super::{guard_subject_holds, proven, written_since};
use crate::codebase::check_facts::{CheckFactMap, CheckFileFacts};
use crate::codebase::dependencies::extract::{
    FiniteIdentifierGuard, GuardKind, GuardSubject, ValueBinding, ValueExpression, ValueFlowFacts,
};
use crate::codebase::postgres::EmbeddedSqlCall;
use crate::codebase::rules::postgres_sql_cross_file::SqlCall;
use crate::codebase::rules::postgres_sql_cross_file::{compile, Bindings, Options, Provenance};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::fx_set;
use std::path::{Path, PathBuf};
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

#[test]
fn guard_subject_matches_only_the_same_binding() {
    let file = PathBuf::from("src/query.ts");
    let mut ts = TsFileFacts::default();
    ts.value_flow.bindings.push(ValueBinding {
        immutable: true,
        owner: None,
        name: "table".to_string(),
        scope: 1,
        declared_at: 4,
        repeated: false,
        invalidated_at: None,
        mutation_contexts: fx_set(),
        value: ValueExpression::Unknown,
    });
    let mut facts = CheckFactMap::default();
    facts.files.push(file.clone());
    facts.ts.insert(
        file.clone(),
        CheckFileFacts {
            ts: Arc::new(ts),
            ..Default::default()
        }
        .into(),
    );
    let bindings = Bindings::new(&facts);
    let flow = ValueFlowFacts::default();
    assert!(holds(
        &flow,
        &bindings,
        &file,
        "table",
        GuardSubject::Binding {
            scope: 1,
            declared_at: 4,
        },
    ));
    assert!(!holds(
        &flow,
        &bindings,
        &file,
        "table",
        GuardSubject::Binding {
            scope: 1,
            declared_at: 9,
        },
    ));
    assert!(!holds(
        &flow,
        &bindings,
        &file,
        "table",
        GuardSubject::Binding {
            scope: 2,
            declared_at: 4,
        },
    ));
    assert!(!holds(
        &flow,
        &bindings,
        &file,
        "missing",
        GuardSubject::Binding {
            scope: 1,
            declared_at: 4,
        },
    ));
    let anonymous = identifier_guard(None);
    assert!(!guard_subject_holds(
        &flow, &bindings, &file, "table", 1, 10, &anonymous,
    ));
    assert!(!written_since(&flow, &anonymous, 10));
}

fn holds(
    flow: &ValueFlowFacts,
    bindings: &Bindings<'_>,
    file: &Path,
    name: &str,
    subject: GuardSubject,
) -> bool {
    guard_subject_holds(
        flow,
        bindings,
        file,
        name,
        1,
        10,
        &identifier_guard(Some(subject)),
    )
}

fn identifier_guard(subject: Option<GuardSubject>) -> FiniteIdentifierGuard {
    FiniteIdentifierGuard {
        context: None,
        name: "table".to_string(),
        from: 0,
        until: 20,
        kind: GuardKind::Assertion {
            callee: "assertWhitelistedSqlIdentifier".to_string(),
            argument: 0,
        },
        subject,
    }
}

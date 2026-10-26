use super::tests::{fixture, prepared, profile};
use super::{
    budget::Limits,
    catalog::PreparedCatalog,
    value::Value,
    values::{Cache, Engine},
};

#[test]
fn boolean_short_branches_are_constrained_and_inherited_members_are_unknown() {
    let files = prepared();
    let path = fixture().join("src/precision.mts");
    let catalog = PreparedCatalog {
        files: &files,
        application: None,
        profile: None,
        resolve: &|_, _| None,
    };
    let source = std::fs::read_to_string(&path).unwrap();
    let mut checked = 0;
    for call in &files
        .get(&path)
        .unwrap()
        .sql_identifier_syntax
        .recovery
        .calls
    {
        let raw = &source[call.span.0 as usize..call.span.1 as usize];
        if !raw.starts_with("read(") || raw.starts_with("read(SQL") {
            continue;
        }
        let mut cache = Cache::default();
        let mut engine = Engine::new(&catalog, &mut cache, Limits::default());
        let values = engine.evaluate(&path, &call.arguments[0]);
        if raw.contains("inherited()") {
            assert!(values.is_none());
            assert!(engine.residual());
        } else {
            let offsets = values
                .unwrap()
                .into_iter()
                .map(|value| {
                    let Value::Sql(text) = value else { panic!() };
                    crate::codebase::postgres::sql_offset_facts(&text.raw.sql)
                        .unwrap()
                        .len()
                })
                .sum::<usize>();
            assert_eq!(offsets > 0, raw.contains("live("), "{raw}");
            assert!(!engine.residual(), "{raw}");
        }
        checked += 1;
    }
    assert_eq!(checked, 5);
}
#[test]
fn executor_identity_members_and_unknown_binding_patterns_keep_evidence_and_uncertainty() {
    let files = prepared();
    let path = fixture().join("src/precision.mts");
    let profile = profile();
    let catalog = PreparedCatalog {
        files: &files,
        application: None,
        profile: Some(&profile),
        resolve: &|_, _| None,
    };
    let source = std::fs::read_to_string(&path).unwrap();
    let models = &files.get(&path).unwrap().sql_identifier_syntax.recovery;
    let mut checked = 0;
    for call in &models.calls {
        let raw = &source[call.span.0 as usize..call.span.1 as usize];
        if ![
            "sameExecutor(",
            "memberCarrier(",
            "logicalCarrier(",
            "consumedMember(",
            "unsupportedAfterSink(",
        ]
        .iter()
        .any(|prefix| raw.starts_with(prefix))
        {
            continue;
        }
        let mut cache = Cache::default();
        let proof = Engine::new(&catalog, &mut cache, Limits::default())
            .transport(&path, call, &models.executors[&profile])
            .unwrap();
        let offsets = proof
            .alternatives
            .iter()
            .map(|sql| {
                crate::codebase::postgres::sql_offset_facts(&sql.sql)
                    .unwrap()
                    .len()
            })
            .sum::<usize>();
        if raw.starts_with("sameExecutor(") {
            assert!(offsets > 0);
            assert!(proof.complete());
        } else if raw.starts_with("consumedMember(") {
            assert!(proof.complete());
        } else {
            assert!(proof.residual, "{raw}");
        }
        if raw.starts_with("unsupportedAfterSink(") {
            assert!(offsets > 0);
        }
        checked += 1;
    }
    assert_eq!(checked, 5);
}

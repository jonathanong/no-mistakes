use super::tests::{fixture, prepared, profile};
use super::{
    budget::Limits,
    catalog::PreparedCatalog,
    value::Value,
    values::{Cache, Engine},
    ApplicationKey,
};
use std::path::Path;

#[test]
fn pure_sql_siblings_survive_failed_members_appends_templates_and_statements() {
    let files = prepared();
    let path = fixture().join("src/partials.mts");
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
        if !raw.starts_with("read(") {
            continue;
        }
        let mut cache = Cache::default();
        let mut engine = Engine::new(&catalog, &mut cache, Limits::default());
        let values = engine.evaluate(&path, &call.arguments[0]).expect(raw);
        assert!(engine.residual(), "{raw}");
        for value in values {
            let sql = match value {
                Value::Sql(text) | Value::Text(text) => text.raw.sql,
                _ => panic!(),
            };
            assert_eq!(
                crate::codebase::postgres::sql_offset_facts(&sql)
                    .unwrap()
                    .len(),
                1,
                "{raw}"
            );
        }
        checked += 1;
    }
    assert_eq!(checked, 4);
}
#[test]
fn generated_bind_placements_are_checked_structurally_and_authored_text_is_literal() {
    let files = prepared();
    let path = fixture().join("src/placements.mts");
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
        if !raw.starts_with("read(") {
            continue;
        }
        let mut cache = Cache::default();
        let values = Engine::new(&catalog, &mut cache, Limits::default())
            .evaluate(&path, &call.arguments[0])
            .expect(raw);
        let Value::Sql(sql) = &values[0] else {
            panic!()
        };
        assert_eq!(sql.valid_binds(), raw.contains("authoredTable"), "{raw}");
        checked += 1;
    }
    assert_eq!(checked, 3);
}
#[test]
fn caller_replay_finds_only_the_actual_sink_and_rejects_closure_mutation() {
    let files = prepared();
    let path = fixture().join("src/replay.mts");
    let helper = fixture().join("src/helpers.mts");
    let profile = profile();
    let resolve =
        |specifier: &str, _: &Path| (specifier == "./helpers.mjs").then(|| helper.clone());
    let catalog = PreparedCatalog {
        files: &files,
        application: None,
        profile: Some(&profile),
        resolve: &resolve,
    };
    let models = &files.get(&path).unwrap().sql_identifier_syntax.recovery;
    let source = std::fs::read_to_string(&path).unwrap();
    let mut checked = 0;
    for call in &models.calls {
        let raw = &source[call.span.0 as usize..call.span.1 as usize];
        if !raw.starts_with("read(") && !raw.starts_with("query(") {
            continue;
        }
        let mut cache = Cache::default();
        let proof = Engine::new(&catalog, &mut cache, Limits::default()).site(
            &path,
            call,
            &models.executors[&profile],
        );
        if raw == "read(statement, options)" {
            assert!(proof.unwrap().complete());
        } else if raw == "read(statement)"
            && source[..call.span.0 as usize]
                .rfind("closureMutation")
                .is_none()
        {
            assert!(!proof.unwrap().alternatives.is_empty());
        } else {
            assert!(proof.is_none(), "{raw}");
        }
        checked += 1;
    }
    assert_eq!(checked, 5);
}
#[test]
fn broader_other_rule_facts_cannot_change_existing_application_calls() {
    let mut files = prepared();
    let profile = profile();
    let a = ApplicationKey::new(&profile, "postgres-no-offset", 0);
    let b = ApplicationKey::new(&profile, "postgres-no-mutable-sql-registry", 0);
    let helper = fixture().join("src/helpers.mts");
    let caller = fixture().join("src/query.mts");
    for (path, file) in &mut files {
        if path != &helper {
            file.sql_identifier_syntax
                .low_applications
                .push((profile.clone(), a.rule.clone(), 0));
        }
    }
    super::project(&mut files, |specifier, _| {
        (specifier == "./helpers.mjs").then(|| helper.clone())
    });
    let before = format!(
        "{:?}",
        files
            .get(&caller)
            .unwrap()
            .sql_identifier_syntax
            .calls_for(&a)
    );
    for (_, file) in &mut files {
        file.sql_identifier_syntax
            .low_applications
            .push((profile.clone(), b.rule.clone(), 0));
    }
    super::project(&mut files, |specifier, _| {
        (specifier == "./helpers.mjs").then(|| helper.clone())
    });
    let syntax = &files.get(&caller).unwrap().sql_identifier_syntax;
    assert_eq!(before, format!("{:?}", syntax.calls_for(&a)));
    assert!(syntax
        .calls_for(&b)
        .unwrap()
        .iter()
        .any(|call| matches!(call.input, super::super::Input::Recovered(_))));
}

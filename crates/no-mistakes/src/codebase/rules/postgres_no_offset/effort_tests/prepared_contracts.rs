use super::*;

#[test]
fn missing_prepared_inputs_fail_while_cached_parser_errors_remain_authoritative() {
    let (cfg, files, sources, mut facts) = prepared(load(".no-mistakes.low-ignore.yml"));
    let path = root().join("src/query.ts");
    let file = facts.ts.remove(&path).unwrap();
    let missing =
        check_with_files_sources_and_facts(&root(), &cfg, &files, &sources, &facts).unwrap_err();
    assert!(missing.to_string().contains("prepared facts missing"));
    facts.ts.insert(path.clone(), file);
    let file = std::sync::Arc::get_mut(facts.ts.get_mut(&path).unwrap()).unwrap();
    file.sql_identifier_syntax.profiles.clear();
    let projection =
        check_with_files_sources_and_facts(&root(), &cfg, &files, &sources, &facts).unwrap_err();
    assert!(projection.to_string().contains("syntax projection missing"));
    for outer in [true, false] {
        let file = std::sync::Arc::get_mut(facts.ts.get_mut(&path).unwrap()).unwrap();
        file.parse_error = outer.then(|| "captured parser error".into());
        std::sync::Arc::make_mut(&mut file.ts).parse_error =
            (!outer).then(|| "captured parser error".into());
        // The prepared parser failure overrides the valid file still on disk.
        let findings =
            check_with_files_sources_and_facts(&root(), &cfg, &files, &sources, &facts).unwrap();
        assert!(findings
            .iter()
            .all(|finding| finding.file != "src/query.ts"));
    }
}

#[test]
fn mixed_invalid_sql_abstains_and_unavailable_max_statements_fall_back_to_literals() {
    let mut cfg = load(".no-mistakes.low-advise.yml");
    cfg.rules[0].options.as_mapping_mut().unwrap().insert(
        "include".into(),
        serde_yaml::from_str("[src/invalid.ts]").unwrap(),
    );
    let mut max = cfg.rules[0].clone();
    max.options
        .as_mapping_mut()
        .unwrap()
        .insert("effort".into(), "max".into());
    cfg.rules.push(max);
    let (mut cfg, files, sources, mut facts) = prepared(cfg);
    cfg.rules.truncate(1);
    facts.enable_sql_verdicts();
    check_with_files_sources_and_facts(&root(), &cfg, &files, &sources, &facts).unwrap();
    assert_eq!(facts.sql_verdicts().len(), 2);
    assert!(facts.sql_verdicts().iter().all(|verdict| verdict.status
        == crate::codebase::postgres::SqlVerdictStatus::Abstain
        && verdict.residual_reasons == ["invalid-sql"]));

    let mut cfg = load(".no-mistakes.low-ignore.yml");
    let mut max = cfg.rules[0].clone();
    max.options
        .as_mapping_mut()
        .unwrap()
        .insert("effort".into(), "max".into());
    cfg.rules.push(max);
    let (mut cfg, files, sources, mut facts) = prepared(cfg);
    cfg.rules.truncate(1);
    // A profile without prepared statement demand must not suppress low analysis.
    facts.postgres.embedded.clear();
    let mut findings =
        check_with_files_sources_and_facts(&root(), &cfg, &files, &sources, &facts).unwrap();
    crate::codebase::rules::suppress_rule_findings_with_sources(&root(), &mut findings, &sources);
    assert_eq!(findings, run(".no-mistakes.low-ignore.yml"));
}

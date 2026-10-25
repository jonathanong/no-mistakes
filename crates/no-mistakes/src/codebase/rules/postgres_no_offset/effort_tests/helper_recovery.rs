use super::*;

#[test]
fn saved_effort_configs_keep_max_default_and_definite_subset() {
    let max = run(".no-mistakes.max.yml");
    assert_eq!(max, run(".no-mistakes.yml"));
    let low = run(".no-mistakes.low.yml");
    let ignore = run(".no-mistakes.low-ignore.yml");
    assert!(
        ignore.iter().all(|finding| max.contains(finding)),
        "low={ignore:#?}, max={max:#?}"
    );
    assert!(max.len() >= low.len(), "low={low:#?}, max={max:#?}");
    // The same-file helper now has a complete literal SQL proof at low effort.
    assert_eq!(ignore.len(), 6, "{ignore:#?}");
    assert!(ignore
        .iter()
        .any(|finding| finding.file == "src/query.ts" && finding.line == 8));
}

#[test]
fn verdicts_and_advisories_preserve_low_demand_and_call_spans() {
    for verdicts in [false, true] {
        let (cfg, files, sources, mut facts) = prepared(load(".no-mistakes.low-advise.yml"));
        if verdicts {
            facts.enable_sql_verdicts();
        }
        check_with_files_sources_and_facts(&root(), &cfg, &files, &sources, &facts).unwrap();
        assert!(facts.ts.values().all(|file| file.embedded_sql.is_empty()));
        let records = facts.sql_verdicts();
        let residuals = records
            .iter()
            .filter(|record| record.advisory)
            .collect::<Vec<_>>();
        assert_eq!(residuals.len(), 2, "{records:#?}");
        let source = sources.read_path(&root().join("src/query.ts")).unwrap();
        let residual_sql = residuals
            .iter()
            .map(|record| &source[record.span.0 as usize..record.span.1 as usize])
            .collect::<Vec<_>>();
        assert_eq!(
            residual_sql,
            ["query(`SELECT 1 ${tail}`)", r"query('SELECT 1\nOFFSET 9')"]
        );
        assert!(residuals.iter().all(|record| record.span.1 > record.span.0));
        assert!(residuals
            .iter()
            .all(|record| record.status == crate::codebase::postgres::SqlVerdictStatus::Abstain));
        if verdicts {
            assert_eq!(records.len(), 10);
            assert!(records.iter().any(|record| record.file == "src/query.ts"
                && &source[record.span.0 as usize..record.span.1 as usize] == "query(build())"
                && record.status == crate::codebase::postgres::SqlVerdictStatus::Violation
                && !record.advisory));
        }
    }
}

use super::*;

// Output cases moved to cli_postgres_snapshot_no_offset.

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

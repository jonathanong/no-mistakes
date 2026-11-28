use super::comment::{parse_reason_text, ReasonStatus};
use super::comments_index::CommentIndex;
use super::*;
use crate::codebase::analysis_session::AnalysisSession;
use crate::config::v2::schema::RuleSeverity;

mod edge_cases;

fn fixture(scenario: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-lock-reason/fixture")
        .join(scenario)
}

#[test]
fn parses_valid_and_incomplete_lock_reason_comments() {
    assert_eq!(
        parse_reason_text("lock-reason: prevents: double refund; writer: refund worker retry"),
        ReasonStatus::Valid
    );
    assert_eq!(
        parse_reason_text("LOCK-REASON: WRITER: job runner; PREVENTS: duplicate claim"),
        ReasonStatus::Valid
    );
    assert_eq!(
        parse_reason_text(
            "*\n * lock-reason:\n * prevents: double debit\n * writer: retry worker\n"
        ),
        ReasonStatus::Valid
    );
    assert_eq!(
        parse_reason_text("lock-reason: writer: refund worker retry"),
        ReasonStatus::MissingPrevents
    );
    assert_eq!(
        parse_reason_text("lock-reason: prevents: ; writer: refund worker retry"),
        ReasonStatus::MissingPrevents
    );
    assert_eq!(
        parse_reason_text("lock-reason: prevents: double refund"),
        ReasonStatus::MissingWriter
    );
    assert_eq!(
        parse_reason_text("lock-reason: prevents: double refund; writer:   "),
        ReasonStatus::MissingWriter
    );
    assert_eq!(
        parse_reason_text("lock-reason: needed for consistency"),
        ReasonStatus::MissingBoth
    );
    assert_eq!(
        parse_reason_text("no-mistakes: deadlock-safe -- ordered by account_id"),
        ReasonStatus::Absent
    );
    assert_eq!(
        parse_reason_text("my-lock-reason: prevents: a; writer: b"),
        ReasonStatus::Absent
    );
    assert_eq!(
        parse_reason_text("lock-reason: reprevents: a; cowriter: b"),
        ReasonStatus::MissingBoth
    );
}

#[test]
fn comment_index_respects_sql_and_ts_boundaries_and_string_literals() {
    let sql = "\
-- lock-reason: prevents: a; writer: b
SELECT id FROM a FOR UPDATE; SELECT id FROM b FOR UPDATE;
SELECT '-- lock-reason: prevents: fake; writer: fake', $$-- lock-reason: prevents: f; writer: w$$ FROM c FOR UPDATE;
/* outer /* nested */ lock-reason: prevents: nested; writer: worker */
SELECT id FROM d FOR UPDATE;
-- lock-reason: prevents: stale; writer: old

SELECT id FROM e FOR UPDATE;
SELECT id FROM f FOR UPDATE; SELECT id FROM g FOR UPDATE -- lock-reason: prevents: g; writer: wg
";
    let index = CommentIndex::for_sql(sql);
    assert_eq!(index.status_for_sql_statement(2, 1, 2), ReasonStatus::Valid);
    assert_eq!(
        index.status_for_sql_statement(2, 30, 2),
        ReasonStatus::Absent
    );
    assert_eq!(
        index.status_for_sql_statement(3, 1, 3),
        ReasonStatus::Absent
    );
    assert_eq!(index.status_for_sql_statement(5, 1, 5), ReasonStatus::Valid);
    assert_eq!(
        index.status_for_sql_statement(8, 1, 8),
        ReasonStatus::Absent
    );
    assert_eq!(
        index.status_for_sql_statement(9, 1, 9),
        ReasonStatus::Absent
    );
    assert_eq!(
        index.status_for_sql_statement(9, 30, 9),
        ReasonStatus::Valid
    );

    let ts = "\
// lock-reason: prevents: dup
// writer: worker
await query(`SELECT ${foo ? `a` : `b`} FROM t FOR UPDATE`);
const s = \"// lock-reason: prevents: fake; writer: fake\";
await query('SELECT 1 FOR UPDATE');
";
    let ts_index = CommentIndex::for_ts(ts);
    assert_eq!(ts_index.status_for_ts_line(3), ReasonStatus::Valid);
    assert_eq!(ts_index.status_for_ts_line(5), ReasonStatus::Absent);
}

#[test]
fn default_severity_is_advisory_and_can_be_raised_to_error() {
    let session = AnalysisSession::new(None);
    let advisory = session
        .config(&fixture("advisory"), Some(Path::new(".no-mistakes.yml")))
        .unwrap();
    assert_eq!(advisory.rules[0].severity, RuleSeverity::Advisory);
    assert!(advisory.rule_is_advisory(RULE_ID));

    let error = session
        .config(&fixture("error"), Some(Path::new(".no-mistakes.yml")))
        .unwrap();
    assert_eq!(error.rules[0].severity, RuleSeverity::Error);
    assert!(!error.rule_is_advisory(RULE_ID));
}

#[test]
fn co_demand_with_lock_inventory_shares_one_embedded_sql_profile() {
    let root = fixture("co-demand");
    let session = AnalysisSession::new(None);
    let both = session
        .config(&root, Some(Path::new(".no-mistakes.yml")))
        .unwrap();
    let only = session
        .config(&root, Some(Path::new("inventory-only.yml")))
        .unwrap();
    let mut plan_both = crate::codebase::check_facts::CheckFactPlan::default();
    crate::codebase::postgres::configure_prepared_postgres_plan(&root, &both, &mut plan_both)
        .unwrap();
    crate::codebase::postgres::assign_prepared_embedded_sql_options(&both, &mut plan_both).unwrap();
    let mut plan_only = crate::codebase::check_facts::CheckFactPlan::default();
    crate::codebase::postgres::configure_prepared_postgres_plan(&root, &only, &mut plan_only)
        .unwrap();
    crate::codebase::postgres::assign_prepared_embedded_sql_options(&only, &mut plan_only).unwrap();
    assert_eq!(plan_both.embedded_sql_options.len(), 1);
    assert_eq!(
        plan_both.embedded_sql_options,
        plan_only.embedded_sql_options
    );
    assert!(plan_both.postgres_lock_inventory);
}

#[test]
fn check_with_files_honors_suppression_and_custom_message_prefix() {
    let root = fixture("error");
    let session = AnalysisSession::new(None);
    let mut config = session
        .config(&root, Some(Path::new(".no-mistakes.yml")))
        .unwrap()
        .as_ref()
        .clone();
    config.rules[0].message = Some("[lock-policy]".into());
    let files = vec![root.join("sql/locks.sql"), root.join("src/app.ts")];
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert_eq!(findings.len(), 5);
    assert!(findings
        .iter()
        .all(|finding| finding.message.starts_with("[lock-policy] ")));
}

#[test]
fn co_demand_does_not_repeat_prepared_sql_projections() {
    let root = fixture("co-demand");
    let count_projections = |config_file: &str| {
        let observer = crate::diagnostics::InvocationObserver::new(true);
        let report = crate::diagnostics::with_observer(Some(Arc::clone(&observer)), || {
            crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(
                serde_json::json!({
                    "root": root,
                    "config": config_file,
                }),
            ))
            .unwrap()
        });
        assert!(!report.is_empty());
        observer
            .snapshot()
            .work
            .get("postgres.prepared.statement_projections")
            .copied()
            .unwrap_or(0)
    };
    let only = count_projections("inventory-only.yml");
    let both = count_projections(".no-mistakes.yml");
    assert_eq!(only, 1);
    assert_eq!(both, only);
}

fn rule_config(options: serde_json::Value) -> NoMistakesConfig {
    serde_json::from_value(serde_json::json!({
        "rules": [{
            "rule": RULE_ID,
            "scope": "repository",
            "severity": "error",
            "options": options,
        }]
    }))
    .unwrap()
}

#[test]
fn embedded_variants_report_truncated_opaque_and_broken_ts_gaps() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/postgres-lock-inventory-core/variants");
    let config = rule_config(serde_json::json!({
        "importSpecifier": "@example/db",
        "executorNames": ["query"],
        "unanalyzable": "report",
    }));
    let files = vec![root.join("broken.ts"), root.join("queries.ts")];
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert!(findings.iter().any(|f| {
        f.file == "broken.ts"
            && f.message
                .contains("TypeScript source could not be analyzed")
    }));
    assert!(findings.iter().any(|f| {
        f.file == "queries.ts"
            && f.message
                .contains("executor SQL alternatives are incomplete")
    }));
    assert!(findings.iter().any(|f| {
        f.file == "queries.ts" && f.message.contains("executor SQL could not be resolved")
    }));
    assert!(findings
        .iter()
        .any(|f| { f.file == "queries.ts" && f.target.as_deref() == Some("for-share") }));
}

#[test]
fn embedded_imported_locks_and_sql_helpers_report_reason_findings() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/postgres-lock-inventory-core/imported");
    let config = rule_config(serde_json::json!({
        "importSpecifier": "@example/db",
        "executorNames": ["query"],
        "helpers": ["reserveJob"],
        "unanalyzable": "report",
    }));
    let files = vec![
        root.join("queries.mts"),
        root.join("alternatives.mts"),
        root.join("lock-clause.mts"),
    ];
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert!(findings
        .iter()
        .any(|f| { f.file == "queries.mts" && f.target.as_deref() == Some("for-update") }));
    assert!(findings
        .iter()
        .any(|f| { f.file == "queries.mts" && f.target.as_deref() == Some("helper") }));
    assert!(findings
        .iter()
        .any(|f| { f.file == "alternatives.mts" && f.target.as_deref() == Some("helper") }));
}

#[test]
fn embedded_routine_and_parse_failure_report_unanalyzable_gaps() {
    let routine_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-lock-inventory/fixture/routine");
    let config = rule_config(serde_json::json!({
        "importSpecifier": "@example/db",
        "executorNames": ["query"],
        "unanalyzable": "report",
    }));
    let routine_findings =
        check_with_files(&routine_root, &config, &[routine_root.join("app.ts")]).unwrap();
    assert!(routine_findings.iter().any(|f| {
        f.file == "app.ts"
            && f.message
                .contains("stored routine body could not be resolved")
    }));
    assert!(routine_findings
        .iter()
        .any(|f| f.file == "app.ts" && f.target.as_deref() == Some("for-share")));

    let gap_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-lock-inventory/fixture/gap-multiplicity");
    let gap_findings = check_with_files(&gap_root, &config, &[gap_root.join("app.ts")]).unwrap();
    assert!(gap_findings.iter().any(|f| {
        f.file == "app.ts" && f.message.contains("SQL could not be parsed completely")
    }));
}

#[test]
fn lexers_cover_escapes_dollar_quotes_templates_and_unterminated_inputs() {
    let sql = concat!(
        "SELECT E'esc\\'\\\\\\nnext', 'it''s\nline', \"col\"\"name\", $1, $plain, 'é',\n",
        "$tag$\n-- lock-reason: prevents: fake; writer: fake\n$tag$\n",
        "FROM jobs FOR UPDATE\n;\n",
        "/* outer\n /* inner */ \n lock-reason: prevents: p; writer: w */\n",
        "SELECT id FROM jobs\n;\n",
        "SELECT $$unterminated",
    );
    let sql_index = CommentIndex::for_sql(sql);
    assert_eq!(
        sql_index.status_for_sql_statement(1, 1, 5),
        ReasonStatus::Absent
    );
    assert_eq!(
        sql_index.status_for_sql_statement(10, 1, 10),
        ReasonStatus::Valid
    );
    assert_eq!(
        CommentIndex::for_sql("/* unterminated").status_for_sql_statement(1, 1, 1),
        ReasonStatus::Absent
    );

    let ts = concat!(
        "/* block\n * lock-reason: prevents: dup; writer: worker */\n",
        "const a = 'esc\\'\\\nline';\n",
        "const b = `tmpl \\` \\\n ${ { x: { y: 1 } } } \n tail`;\n",
        "const c = 'é';\n",
        "const d = \"unterminated\n",
    );
    let ts_index = CommentIndex::for_ts(ts);
    assert_eq!(ts_index.status_for_ts_line(3), ReasonStatus::Valid);
    assert_eq!(ts_index.status_for_ts_line(5), ReasonStatus::Absent);
    assert_eq!(
        CommentIndex::for_ts("const u = `unterminated").status_for_ts_line(1),
        ReasonStatus::Absent
    );
    assert_eq!(
        CommentIndex::for_ts("/* unterminated").status_for_ts_line(1),
        ReasonStatus::Absent
    );
}

#[test]
fn reason_status_merge_and_multi_tag_parsing_cover_all_branches() {
    assert_eq!(
        parse_reason_text("lock-reason: incomplete\nlock-reason: prevents: dup; writer: worker"),
        ReasonStatus::Valid
    );
    assert_eq!(
        ReasonStatus::Absent.merge(ReasonStatus::MissingBoth),
        ReasonStatus::MissingBoth
    );
    assert_eq!(
        ReasonStatus::MissingBoth.merge(ReasonStatus::Absent),
        ReasonStatus::MissingBoth
    );
    assert_eq!(
        ReasonStatus::MissingBoth.merge(ReasonStatus::MissingWriter),
        ReasonStatus::MissingWriter
    );
    assert_eq!(
        ReasonStatus::MissingWriter.merge(ReasonStatus::MissingBoth),
        ReasonStatus::MissingWriter
    );
    assert_eq!(
        ReasonStatus::MissingWriter.merge(ReasonStatus::MissingPrevents),
        ReasonStatus::MissingPrevents
    );
    assert_eq!(
        ReasonStatus::MissingPrevents.merge(ReasonStatus::MissingWriter),
        ReasonStatus::MissingPrevents
    );
    assert_eq!(
        ReasonStatus::MissingPrevents.merge(ReasonStatus::Valid),
        ReasonStatus::Valid
    );
    assert_eq!(
        ReasonStatus::Valid.merge(ReasonStatus::Absent),
        ReasonStatus::Valid
    );
    for status in [
        ReasonStatus::Absent,
        ReasonStatus::MissingBoth,
        ReasonStatus::MissingPrevents,
        ReasonStatus::MissingWriter,
        ReasonStatus::Valid,
    ] {
        assert!(!comment::reason_message("a.sql", 1, "for-update", status).is_empty());
    }
}

#[test]
fn findings_and_options_cover_undeclared_executors_legacy_policy_and_unreadable_sql() {
    use crate::codebase::postgres::{SqlFunctionCallFact, UndeclaredExecutorType};
    use crate::codebase::rules::validate_options::collect_config_errors as collect;
    use crate::config::v2::Unanalyzable;

    let mut out = Vec::new();
    let undeclared = vec![
        UndeclaredExecutorType {
            type_name: "Db".into(),
            module: "@example/db".into(),
            line: 2,
            span: (0, 1),
            receiver_name: "db".into(),
            owner: None,
            explicit_annotation: true,
            configured_type: true,
            known_type: false,
        },
        UndeclaredExecutorType {
            type_name: "Pool".into(),
            module: "pg".into(),
            line: 3,
            span: (0, 1),
            receiver_name: "pool".into(),
            owner: None,
            explicit_annotation: true,
            configured_type: false,
            known_type: true,
        },
        UndeclaredExecutorType {
            type_name: String::new(),
            module: String::new(),
            line: 4,
            span: (0, 1),
            receiver_name: "tx".into(),
            owner: None,
            explicit_annotation: false,
            configured_type: false,
            known_type: false,
        },
    ];
    findings::push_undeclared_executors("app.ts", &undeclared, &mut out);
    assert_eq!(out.len(), 3);
    assert!(out[0].message.contains("configured executor receiver"));
    assert!(out[1].message.contains("undeclared executor type"));
    assert!(out[2].message.contains("has no declared executor type"));

    let compiled = CompiledOptions::from_rule(
        &rule_config(serde_json::json!({
            "trustedSqlTags": [{ "module": "@example/sql", "name": "sql" }],
            "helpers": ["lock_row"],
        }))
        .rules[0],
    )
    .unwrap();
    assert!(compiled.scans_executors());
    assert!(!findings::matches_sql_helper(
        &compiled,
        &SqlFunctionCallFact {
            name_parts: Vec::new(),
            line: 1,
            column: 1,
            clause: None,
        }
    ));

    for (raw, expected) in [
        ("report", Unanalyzable::Report),
        ("fail", Unanalyzable::Report),
        ("advise", Unanalyzable::Advise),
        ("ignore", Unanalyzable::Ignore),
    ] {
        let cfg = rule_config(serde_json::json!({ "unanalyzableSql": raw }));
        assert_eq!(resolve_unanalyzable(&cfg, &cfg.rules[0]).unwrap(), expected);
    }

    let invalid = rule_config(serde_json::json!({
        "helpers": ["  "],
        "unanalyzableSql": "bogus",
    }));
    let mut errors = Vec::new();
    collect(Path::new("."), None, &invalid, RULE_ID, collect_application_errors, &mut errors);
    assert_eq!(errors.len(), 2);

    let root = fixture("error");
    let cfg = rule_config(serde_json::json!({
        "executorNames": [],
        "sqlInclude": ["sql/**/*.sql"],
        "unanalyzable": "report",
    }));
    let missing_findings = check_with_files(&root, &cfg, &[root.join("sql/missing.sql")]).unwrap();
    assert!(missing_findings
        .iter()
        .any(|f| f.message.contains("SQL source could not be read")));
}

#[test]
fn project_scope_with_schema_catalog_path_loads_and_checks() {
    let root = fixture("project-scope");
    let session = AnalysisSession::new(None);
    let config = session
        .config(&root, Some(Path::new(".no-mistakes.yml")))
        .unwrap();
    let files = vec![root.join("sql/locks.sql"), root.join("src/app.ts")];
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "sql/locks.sql");
    assert_eq!(findings[0].line, 4);
}

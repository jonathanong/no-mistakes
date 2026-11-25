use super::comment::{parse_reason_text, ReasonStatus};
use super::comments_index::CommentIndex;
use super::*;
use crate::codebase::analysis_session::AnalysisSession;
use crate::config::v2::schema::RuleSeverity;

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

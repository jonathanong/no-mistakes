use super::super::comment::ReasonStatus;
use super::super::comments_index::CommentIndex;
use super::super::RULE_ID;
use crate::config::v2::schema::{RuleDef, RuleSeverity};

#[test]
fn ts_statement_bounds_exclude_reasons_from_other_same_line_statements() {
    let ts = "other(/* lock-reason: prevents: dup; writer: worker */); takeLock();\n";
    let index = CommentIndex::for_ts(ts);
    let take_lock_offset = ts.find("takeLock").unwrap() as u32;
    assert_eq!(
        index.status_for_ts_offset(1, take_lock_offset),
        ReasonStatus::Absent
    );

    let ts_same_stmt = "other(); takeLock(/* lock-reason: prevents: dup; writer: worker */);\n";
    let idx_same = CommentIndex::for_ts(ts_same_stmt);
    assert_eq!(
        idx_same.status_for_ts_offset(1, ts_same_stmt.find("takeLock").unwrap() as u32),
        ReasonStatus::Valid
    );

    let ts_later_stmt = "takeLock(); other(/* lock-reason: prevents: dup; writer: worker */);\n";
    let idx_later = CommentIndex::for_ts(ts_later_stmt);
    assert_eq!(idx_later.status_for_ts_offset(1, 0), ReasonStatus::Absent);

    let ts_trailing = "other(); takeLock(); // lock-reason: prevents: dup; writer: worker\n";
    let idx_trailing = CommentIndex::for_ts(ts_trailing);
    assert_eq!(
        idx_trailing.status_for_ts_offset(1, ts_trailing.find("takeLock").unwrap() as u32),
        ReasonStatus::Valid
    );
    assert_eq!(
        idx_trailing.status_for_ts_offset(1, 999),
        ReasonStatus::Valid
    );
}

#[test]
fn sql_helper_without_semicolon_scans_to_statement_end() {
    let sql = concat!(
        "SELECT take_lock(\n",
        "  /* lock-reason: prevents: duplicates; writer: worker */\n",
        "  1\n",
        ")",
    );
    let index = CommentIndex::for_sql(sql);
    assert_eq!(index.status_for_sql_line(1, 8), ReasonStatus::Valid);
}

#[test]
fn dollar_sign_inside_identifier_does_not_start_dollar_quote() {
    let sql = concat!(
        "SELECT foo$tag$ FROM jobs FOR UPDATE;\n",
        "-- lock-reason: prevents: duplicates; writer: worker\n",
        "SELECT id FROM accounts FOR UPDATE;\n",
    );
    let index = CommentIndex::for_sql(sql);
    assert_eq!(index.status_for_sql_statement(3, 1, 3), ReasonStatus::Valid);
}

#[test]
fn multibyte_utf8_in_sql_literals_preserves_character_columns() {
    let quoted = "SELECT 'é';SELECT /* lock-reason: prevents: duplicate; writer: worker */ * FROM jobs FOR UPDATE;";
    let idx_quoted = CommentIndex::for_sql(quoted);
    assert_eq!(
        idx_quoted.status_for_sql_statement(1, 12, 1),
        ReasonStatus::Valid
    );

    let dollar = "SELECT $$é$$;SELECT /* lock-reason: prevents: duplicate; writer: worker */ * FROM jobs FOR UPDATE;";
    let idx_dollar = CommentIndex::for_sql(dollar);
    assert_eq!(
        idx_dollar.status_for_sql_statement(1, 14, 1),
        ReasonStatus::Valid
    );
}

#[test]
fn ts_regex_literals_with_quotes_do_not_swallow_following_reasons() {
    let ts = concat!(
        "const apostrophe = /'/;\n",
        "const cls = /[/\\']+/g;\n",
        "const div = a / b;\n",
        "function f() { return /'/; }\n",
        "// lock-reason: prevents: duplicates; writer: worker\n",
        "takeLock();\n",
    );
    let index = CommentIndex::for_ts(ts);
    let offset = ts.find("takeLock").unwrap() as u32;
    assert_eq!(index.status_for_ts_offset(6, offset), ReasonStatus::Valid);
    assert_eq!(
        CommentIndex::for_ts("const bad = /abc\nconst x = /é").status_for_ts_line(2),
        ReasonStatus::Absent
    );
}

#[test]
fn rule_def_round_trip_preserves_explicit_error_severity_for_lock_reason() {
    let rule: RuleDef = serde_yaml::from_str(&format!(
        "rule: {RULE_ID}\nscope: repository\nseverity: error\n"
    ))
    .unwrap();
    assert_eq!(rule.severity, RuleSeverity::Error);
    let serialized = serde_yaml::to_string(&rule).unwrap();
    let round_tripped: RuleDef = serde_yaml::from_str(&serialized).unwrap();
    assert_eq!(round_tripped.severity, RuleSeverity::Error);
}

#[test]
fn repeated_labels_and_multiple_tags_cannot_satisfy_empty_fields() {
    use super::super::comment::parse_reason_text;
    assert_eq!(
        parse_reason_text(
            "lock-reason: prevents: duplicates; writer:\nlock-reason: prevents: ; writer: worker"
        ),
        ReasonStatus::MissingPrevents
    );
    assert_eq!(
        parse_reason_text("lock-reason: prevents: ; prevents: ; writer: worker"),
        ReasonStatus::MissingPrevents
    );
    assert_eq!(
        parse_reason_text(
            "lock-reason: prevents: ;\nlock-reason: prevents: dup; writer: worker"
        ),
        ReasonStatus::Valid
    );
}

#[test]
fn multiline_ts_calls_and_sql_suffix_after_semicolon_find_reasons() {
    let multiline = concat!(
        "takeLock(\n",
        "  // note\n",
        "  /* lock-reason: prevents: duplicates; writer: worker */\n",
        "  wrap(\"a)b\"),\n",
        ");\n",
    );
    let idx = CommentIndex::for_ts(multiline);
    assert_eq!(idx.status_for_ts_offset(1, 0), ReasonStatus::Valid);

    let unclosed = "takeLock(\n  id\n";
    assert_eq!(
        CommentIndex::for_ts(unclosed).status_for_ts_offset(1, 0),
        ReasonStatus::Absent
    );

    let separate = concat!(
        "takeLock(\n",
        "  id,\n",
        ");\n",
        "other(/* lock-reason: prevents: duplicates; writer: worker */);\n",
    );
    let idx_sep = CommentIndex::for_ts(separate);
    assert_eq!(idx_sep.status_for_ts_offset(1, 0), ReasonStatus::Absent);

    let no_paren = "takeLock; other(/* lock-reason: prevents: dup; writer: w */);\n";
    let idx_np = CommentIndex::for_ts(no_paren);
    assert_eq!(idx_np.status_for_ts_offset(1, 0), ReasonStatus::Absent);
    assert_eq!(idx_np.status_for_ts_position(1, 1), ReasonStatus::Absent);

    let sql = concat!(
        "SELECT 1; SELECT /* lock-reason: prevents: duplicates; writer: worker */\n",
        "  take_lock();\n",
    );
    let sql_idx = CommentIndex::for_sql(sql);
    assert_eq!(sql_idx.status_for_sql_line(2, 3), ReasonStatus::Valid);
}

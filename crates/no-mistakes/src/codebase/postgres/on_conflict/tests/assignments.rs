#[test]
fn after_update_without_where_re_fires_trigger() {
    let sql = "INSERT INTO items (id, note) VALUES (1, 'a')
         ON CONFLICT (id) DO UPDATE SET note = EXCLUDED.note;";
    let file = extract_sql_statement_facts(sql);
    let triggers = extract_sql_statement_facts(
        "CREATE TRIGGER t AFTER UPDATE ON items FOR EACH ROW EXECUTE FUNCTION audit();",
    )
    .triggers;
    let found = judge_file(&file, &catalog(&[], &triggers, &[], &[]));
    assert!(
        found.iter().any(|(_, message)| message.contains("re-fire")),
        "{found:?}"
    );
}

#[test]
fn statement_level_delete_trigger_is_ignored() {
    let sql = "INSERT INTO items (id, note) VALUES (1, 'a')
         ON CONFLICT (id) DO UPDATE SET note = EXCLUDED.note;";
    let file = extract_sql_statement_facts(sql);
    let triggers = extract_sql_statement_facts(
        "CREATE TRIGGER t AFTER DELETE ON items EXECUTE FUNCTION audit();",
    )
    .triggers;
    assert!(judge_file(&file, &catalog(&[], &triggers, &[], &[])).is_empty());
}

#[test]
fn allowlisted_trigger_writing_arbiter_is_unsafe() {
    let sql = "INSERT INTO items (id) VALUES (1) ON CONFLICT (id) DO NOTHING;";
    let file = extract_sql_statement_facts(sql);
    let triggers = extract_sql_statement_facts(
        "CREATE TRIGGER t BEFORE INSERT ON items FOR EACH ROW EXECUTE FUNCTION audit();",
    )
    .triggers;
    let replay_safe = ["audit".to_string()];
    let writes = [("audit".to_string(), vec!["id".to_string()])];
    let found = judge_file(&file, &catalog(&[], &triggers, &replay_safe, &writes));
    assert!(
        found
            .iter()
            .any(|(_, message)| message.contains("conflict-arbiter")),
        "{found:?}"
    );
}

fn assert_message(sql: &str, needle: &str) {
    let found = messages(sql);
    assert!(
        found.iter().any(|message| message.contains(needle)),
        "{found:?}"
    );
}

#[test]
fn unproven_assignment_forms_are_judged() {
    assert_message(
        "INSERT INTO items (id, note) VALUES (1, 'a')
         ON CONFLICT (id) DO UPDATE SET note = (SELECT 'x');",
        "subquery",
    );
    assert_message(
        "INSERT INTO items (id, n) VALUES (1, 0)
         ON CONFLICT (id) DO UPDATE SET n = n + 1;",
        "non-convergent",
    );
    assert!(messages(
        "INSERT INTO items (id, score) VALUES (1, 0)
         ON CONFLICT (id) DO UPDATE SET score = GREATEST(score, EXCLUDED.score, NULL);"
    )
    .is_empty());
    assert_message(
        "INSERT INTO items (id, seen) VALUES (1, now())
         ON CONFLICT (id) DO UPDATE SET seen = LEAST(now(), seen);",
        "GREATEST/LEAST",
    );
    assert_message(
        "INSERT INTO items (id, score) VALUES (1, 0)
         ON CONFLICT (id) DO UPDATE SET score = GREATEST(other, 1);",
        "not convergent",
    );
    assert_message(
        "INSERT INTO items (id, seen) VALUES (1, now())
         ON CONFLICT (id) DO UPDATE SET seen = COALESCE(now(), items.seen);",
        "may only follow",
    );
    assert_message(
        "INSERT INTO items (id, seen) VALUES (1, now())
         ON CONFLICT (id) DO UPDATE SET seen = COALESCE(note, items.seen);",
        "different column",
    );
}

#[test]
fn disabled_convergence_and_volatility_ignore_unproven_forms() {
    let file = extract_sql_statement_facts(
        "INSERT INTO items (id, note, seen) VALUES (1, 'a', now())
         ON CONFLICT (id) DO UPDATE SET note = (SELECT 'x'), seen = GREATEST(now(), other),
           n = n + 1, updated_at = created_at;",
    );
    let mut options = catalog(&[], &[], &[], &[]);
    options.check_convergence = false;
    options.check_volatility = false;
    options.check_arbiter = false;
    options.check_triggers = false;
    options.check_generated = false;
    assert!(judge_file(&file, &options).is_empty());
}

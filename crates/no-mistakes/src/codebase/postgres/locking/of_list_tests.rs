use super::{extract_locking_select_metadata, LockingSelectMetadata};

const FROM: &str = "FROM accounts a JOIN orders o ON o.account_id = a.id WHERE a.id = ANY($1)";

fn locks(clause: &str) -> Vec<LockingSelectMetadata> {
    extract_locking_select_metadata(&format!("SELECT 1 {FROM} ORDER BY a.id {clause}"))
        .unwrap_or_else(|error| panic!("{clause}: {error}"))
}

fn tables(clause: &str) -> Vec<String> {
    let locks = locks(clause);
    assert_eq!(locks.len(), 1, "{clause}");
    locks[0].tables.clone().expect("resolved tables")
}

#[test]
fn multi_target_of_list_matches_the_split_form() {
    let split = locks("FOR UPDATE OF a FOR UPDATE OF o");
    assert_eq!(locks("FOR UPDATE OF a, o"), split);
    assert_eq!(tables("FOR UPDATE OF a, o"), ["accounts", "orders"]);
    assert_eq!(tables("for update of a ,o , a"), ["accounts", "orders"]);
}

#[test]
fn multi_target_of_list_with_share_is_not_a_for_update_lock() {
    assert!(locks("FOR SHARE OF a, o").is_empty());
}

#[test]
fn sqlparser_rejects_the_key_strengths_with_or_without_a_list() {
    // sqlparser only knows UPDATE and SHARE, so these fail the same way for a
    // single target; the list rewrite must not change that.
    for strength in ["NO KEY UPDATE", "KEY SHARE"] {
        for target in ["a", "a, o"] {
            let sql = format!("SELECT 1 {FROM} FOR {strength} OF {target}");
            assert!(extract_locking_select_metadata(&sql).is_err(), "{sql}");
        }
    }
}

#[test]
fn multi_target_of_list_keeps_wait_policies() {
    assert!(locks("FOR UPDATE OF a, o SKIP LOCKED")[0].skips_locked_rows);
    assert!(!locks("FOR UPDATE OF a, o NOWAIT")[0].skips_locked_rows);
    assert_eq!(tables("FOR UPDATE OF a, o NOWAIT"), ["accounts", "orders"]);
}

#[test]
fn multi_target_of_list_accepts_qualified_and_quoted_names() {
    let sql = "SELECT 1 FROM public.accounts AS a JOIN public.orders o ON true \
               WHERE a.id = ANY($1) ORDER BY a.id FOR UPDATE OF public.accounts, \"o\"";
    let locks = extract_locking_select_metadata(sql).expect("parse");
    assert_eq!(
        locks[0].tables.as_deref(),
        Some(&["public.accounts".to_string(), "public.orders".to_string()][..])
    );
}

#[test]
fn unresolved_multi_target_name_fails_closed() {
    assert!(locks("FOR UPDATE OF a, missing")[0].tables.is_none());
}

#[test]
fn malformed_of_lists_are_not_rewritten() {
    for clause in [
        "FOR UPDATE OF a,",
        "FOR UPDATE OF a, o.",
        "FOR UPDATE OF a, (o)",
        "FOR UPDATE OF",
    ] {
        let sql = format!("SELECT 1 {FROM} {clause}");
        assert!(extract_locking_select_metadata(&sql).is_err(), "{clause}");
    }
}

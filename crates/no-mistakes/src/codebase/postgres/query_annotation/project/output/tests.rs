
use super::*;

#[test]
fn exported_sql_requires_every_joined_branch_to_have_known_text() {
    let first = Value::Prefix("/* a */ SELECT 1".into(), true, None);
    let second = Value::Prefix("/* b */ SELECT 2".into(), true, None);
    let known = Value::Joined(vec![first.clone(), second.clone(), first].into());
    let projected = projected_sql(&known).expect("all branches have SQL");
    assert_eq!(projected.len(), 2, "duplicate branches are one alternative");
    assert_eq!(projected[0].text, "/* a */ SELECT 1");
    assert_eq!(projected[1].text, "/* b */ SELECT 2");
    assert!(projected.iter().all(|branch| branch.complete));

    let uncertain = Value::Joined(vec![second, Value::Unknown].into());
    assert!(projected_sql(&uncertain).is_none());
}

#[test]
fn authored_placeholder_spelling_is_plain_sql_text() {
    let authored = Value::Evaluated(
        Box::new(Value::Prefix("SELECT sql_placeholder_1".into(), true, None)),
        false,
    );
    let projected = projected_sql(&authored).expect("literal SQL");
    assert!(projected[0].complete);
    assert!(!projected[0].only_value_placeholders);
    assert!(projected_sql(&Value::Promise(Box::new(authored))).is_none());
}

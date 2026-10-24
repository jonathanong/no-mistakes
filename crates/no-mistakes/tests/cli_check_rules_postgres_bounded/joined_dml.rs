use super::{check_in, findings_of, fixture};

#[test]
fn dml_join_bounds_follow_the_target_key_and_do_not_leak_to_siblings() {
    let root = fixture().join("joined-dml");
    let mut actual = findings_of(check_in(&root, ".no-mistakes.yml"));
    actual.sort();
    assert_eq!(
        actual,
        [
            (
                "index-guards.sql".to_string(),
                2,
                "table:placeholder_partial".to_string(),
            ),
            (
                "index-guards.sql".to_string(),
                3,
                "table:placeholder_expression".to_string(),
            ),
            (
                "index-guards.sql".to_string(),
                6,
                "table:nulls_plain".to_string(),
            ),
            (
                "joined-dml.sql".to_string(),
                2,
                "table:joined_targets".to_string(),
            ),
            (
                "joined-dml.sql".to_string(),
                3,
                "table:joined_targets".to_string(),
            ),
            (
                "joined-dml.sql".to_string(),
                13,
                "table:joined_targets".to_string(),
            ),
        ]
    );
}

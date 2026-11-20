use super::check_json_snapshot::{assert_saved_fixture_report, rules_path};

const RULE: &str = "postgres-require-query-annotation";

#[test]
fn postgres_destructured_option_member_check_json_snapshot() {
    let cases = [(
        "destructured-option-member",
        rules_path(RULE, "fixture", "destructured-option-member"),
        rules_path(RULE, "configs", "destructured-option-member.yml"),
    )];
    let document =
        assert_saved_fixture_report("postgres_destructured_option_member", &cases, &[], &[]);
    let reported: Vec<_> = document["destructured-option-member"]["report"]["rules"]
        .as_array()
        .unwrap()
        .iter()
        .map(|finding| {
            let message = finding["message"].as_str().unwrap();
            assert!(
                message.contains("must start with a /* name */"),
                "{message}"
            );
            finding["sourceLine"].as_str().unwrap()
        })
        .collect();
    assert_eq!(
        reported,
        [
            "const { rows = [] } = await run(sql`SELECT 5`)",
            "const { rows } = await run(sql`SELECT 2`)",
            "const [first] = await run('SELECT 3')",
            "const result = await run(sql`SELECT 4`)",
            "const { rows } = await run(sql`${x} SELECT 1`)",
        ]
    );
}

#[test]
fn postgres_destructured_call_parity_check_json_snapshot() {
    let cases = [(
        "destructured-call-parity",
        rules_path(RULE, "fixture", "destructured-call-parity"),
        rules_path(RULE, "configs", "destructured-call-parity.yml"),
    )];
    let document =
        assert_saved_fixture_report("postgres_destructured_call_parity", &cases, &[], &[]);
    let lines: Vec<_> = document["destructured-call-parity"]["report"]["rules"]
        .as_array()
        .unwrap()
        .iter()
        .map(|finding| finding["line"].as_u64().unwrap())
        .collect();
    assert_eq!(lines, [27, 32, 75, 80]);
}

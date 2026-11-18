use super::*;

#[test]
fn unsupported_parameter_members_keep_all_guard_combinations() {
    let calls = calls("variants-member-guard-edges.ts");
    let cases = [
        "optional parameter",
        "defaulted parameter",
        "destructured parameter",
        "optional property",
        "computed property signature",
        "non-string union member",
        "open alias",
        "generic alias",
        "qualified namespace alias",
        "alias above the variant budget",
        "generic parameter shadowing a closed alias",
        "decorated parameter",
        "later block alias shadowing a closed alias",
        "receiver method can mutate the guarded member",
    ];
    assert_eq!(calls.len(), cases.len());

    for (call, description) in calls.iter().zip(cases) {
        let mut variants = texts(call);
        variants.sort_unstable();
        assert_eq!(
            variants,
            [
                "SELECT 1",
                "SELECT 1 /* mine */",
                "SELECT 1 /* mine */ /* team */",
                "SELECT 1 /* team */",
            ],
            "{description}"
        );
        assert!(!call.variants_truncated(), "{description}");
    }
}

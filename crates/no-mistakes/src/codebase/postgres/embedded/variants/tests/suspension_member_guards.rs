use super::*;
use std::collections::BTreeSet;

#[test]
fn suspension_opens_member_reads_but_keeps_captured_immutable_values() {
    let calls = calls("variants-suspension-member-guards.ts");
    assert_eq!(calls.len(), 7);
    for (index, description) in [
        (0, "member after await"),
        (1, "member after yield"),
        (4, "member alias captured after await"),
    ] {
        let variants: BTreeSet<_> = texts(&calls[index]).into_iter().collect();
        assert_eq!(
            variants,
            BTreeSet::from([
                "SELECT 1",
                "SELECT 1 /* mine */",
                "SELECT 1 /* team */",
                "SELECT 1 /* mine */ /* team */",
            ]),
            "{description}"
        );
        assert!(!calls[index].variants_truncated(), "{description}");
    }
    for (index, description) in [
        (2, "immutable alias before await"),
        (3, "immutable alias before yield"),
        (5, "guards execute before awaiting the query"),
        (6, "non-null asserted stable receiver"),
    ] {
        let variants: BTreeSet<_> = texts(&calls[index]).into_iter().collect();
        assert_eq!(
            variants,
            BTreeSet::from([
                "SELECT 1 /* mine */",
                "SELECT 1 /* team */",
                "SELECT 1 /* mine */ /* team */",
            ]),
            "{description}"
        );
        assert!(!calls[index].variants_truncated(), "{description}");
    }
}

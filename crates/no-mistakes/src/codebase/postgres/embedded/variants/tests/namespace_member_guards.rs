use super::*;

#[test]
fn merged_namespace_exports_block_outer_finite_member_aliases() {
    let calls = calls("variants-namespace-member-guards.ts");
    assert_eq!(calls.len(), 21);
    for index in [0, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 16, 18, 20] {
        let mut variants = texts(&calls[index]);
        variants.sort_unstable();
        assert_eq!(
            variants,
            [
                "SELECT id FROM items",
                "SELECT id FROM items ORDER BY id",
                "SELECT id FROM items WHERE a = 1",
                "SELECT id FROM items WHERE a = 1 ORDER BY id",
            ],
            "namespace call {index}"
        );
        assert!(!calls[index].variants_truncated());
    }
}

#[test]
fn namespace_inventory_preserves_unshadowed_and_own_finite_aliases() {
    let calls = calls("variants-namespace-member-guards.ts");
    assert_eq!(calls.len(), 21);
    for index in [1, 2, 3, 14, 17, 19] {
        let mut variants = texts(&calls[index]);
        variants.sort_unstable();
        assert_eq!(
            variants,
            [
                "SELECT id FROM items ORDER BY id",
                "SELECT id FROM items WHERE a = 1",
                "SELECT id FROM items WHERE a = 1 ORDER BY id",
            ],
            "namespace call {index}"
        );
        assert!(!calls[index].variants_truncated());
    }
}

#[test]
fn unsupported_namespace_owners_do_not_poison_module_aliases() {
    let calls = calls("variants-namespace-member-guards.ts");
    assert_eq!(calls.len(), 21);
    let mut variants = texts(&calls[15]);
    variants.sort_unstable();
    assert_eq!(
        variants,
        [
            "SELECT id FROM items ORDER BY id",
            "SELECT id FROM items WHERE a = 1",
            "SELECT id FROM items WHERE a = 1 ORDER BY id",
        ]
    );
    assert!(!calls[15].variants_truncated());
}

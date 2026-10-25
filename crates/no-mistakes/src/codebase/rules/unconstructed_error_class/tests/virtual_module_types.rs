use super::*;

#[test]
fn tsconfig_visible_virtual_modules_prove_declared_bases_only() {
    let root = fixture()
        .parent()
        .unwrap()
        .join("virtual-module-types/fixture");
    let report = diagnostic_report(&root);
    assert_eq!(
        reported_channel(&report, "rules"),
        [
            "src/generated.ts:3 UnusedGenerated",
            "src/local.ts:3 UnusedLocal",
            "src/runtime.ts:4 UnusedRuntime",
            "src/runtime.ts:8 UnusedDirect",
        ],
        "{report}"
    );
    assert_eq!(
        reported_channel(&report, "advisories"),
        [
            "src/ambiguous.ts:2 unanalyzable",
            "src/missing.ts:2 unanalyzable",
        ],
        "{report}"
    );
}

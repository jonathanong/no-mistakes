use std::path::PathBuf;
use std::process::Command;

#[test]
fn playwright_human_report_does_not_claim_complete_coverage_for_uncertain_ownership() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/playwright-configs/uncertain-coverage/fixture");
    let fixture = crate::saved_fixture::materialize_path(&source);
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["playwright", "--root"])
        .arg(fixture.path())
        .arg("check")
        .output()
        .unwrap();
    let text = String::from_utf8(output.stdout).unwrap();
    assert_eq!(output.status.code(), Some(1), "{text}");
    assert!(
        text.contains("Routes: 1\nCovered routes: 1\nUncovered routes: 0"),
        "{text}"
    );
    assert!(text.contains("Uncovered selectors: 0"), "{text}");
    assert!(
        text.contains("Warning: playwright.config.ts:5: Playwright testMatch"),
        "{text}"
    );
    assert!(
        text.contains("Coverage is uncertain; see warnings above."),
        "{text}"
    );
    assert!(
        !text.contains("All routes and selectors covered."),
        "{text}"
    );
}

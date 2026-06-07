1. Modify `crates/no-mistakes/src/playwright/playwright_urls/tests/extract.rs` using `replace_with_git_merge_diff`
   - Update imports to include `extract_playwright_url_literals_from_program`.
   - Add a test case `extracts_playwright_url_literals_from_program` that uses `crate::playwright::ast::with_program` to generate the `Program` and passes it along with `source` and `navigation_helpers`. The test should verify that the returned URLs are deduplicated, sorted, and properly extract from the program using both built-in (e.g. `page.goto`) and custom helpers.
2. Verify the changes
   - Run `cargo +nightly test -p no-mistakes`
3. Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.

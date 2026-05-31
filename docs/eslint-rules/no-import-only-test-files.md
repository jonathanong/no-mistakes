# `no-mistakes/no-import-only-test-files`

Disallows aggregate test files that only import other tests.

Why: import-only test files obscure real ownership and can distort targeted test
selection.

Counterexample: a test file containing only `import "./users.test"`.

Fix: move tests into the file directly or run the imported test files instead.

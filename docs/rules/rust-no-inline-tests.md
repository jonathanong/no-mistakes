# `rust-no-inline-tests`

Bans inline Rust `#[cfg(test)] mod tests` modules.

```yaml
rules:
  - rule: rust-no-inline-tests
    scope: repository
```

Counterexample: tests embedded at the bottom of `src/lib.rs`.

Fix: move tests into sibling `tests.rs` or integration tests, using fixture
files under `fixtures/` or `test-cases/`.

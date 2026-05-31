# `rust-no-inline-allows`

Bans inline Rust `allow` attributes.

```yaml
rules:
  - rule: rust-no-inline-allows
    scope: repository
```

Counterexample: `#[allow(dead_code)]` above a function.

Fix: remove the allow by addressing the lint or use a documented broader policy
where appropriate.

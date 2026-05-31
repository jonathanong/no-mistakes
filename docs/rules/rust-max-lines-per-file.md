# `rust-max-lines-per-file`

Caps Rust source and test file length.

```yaml
rules:
  - rule: rust-max-lines-per-file
    scope: repository
    options:
      srcMax: 200
      testMax: 500
```

Counterexample: a 900-line Rust source file.

Fix: extract cohesive modules or move test fixtures out of inline code.

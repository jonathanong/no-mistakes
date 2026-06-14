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

Counterexample: a 900-line Rust source file. Blank lines and comments are ignored, but Rust string and raw-string literal contents still count as code lines because they are part of the file body.

Fix: extract cohesive modules or move test fixtures out of inline code.

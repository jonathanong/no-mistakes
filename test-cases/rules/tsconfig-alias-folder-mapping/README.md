# tsconfig-alias-folder-mapping fixtures

`fixture/` is loaded by the rule's Rust unit tests. It covers directory, exact
file, index file, compound-extension, and wildcard-extension targets; real
mapping mismatches; existence checks; multiple roots; and JSONC source locations
through duplicate keys, Unicode escapes, child overrides, and extends arrays.
`valid-files/.no-mistakes.yml` also supports CLI and Node API parity checks.

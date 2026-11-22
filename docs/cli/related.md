# `no-mistakes related`

> Deprecated. Use [`dependents`](dependents.md) for new calls.

This compatibility alias accepts the same options and produces the same reports
and exit codes as `dependents`. Executing it writes one deprecation warning to
stderr; stdout is unchanged, including `--format json`.

```sh
no-mistakes dependents src/value.mts --format json
```

The async Node API `related(options)` remains available with an `@deprecated`
tag. Use `dependents(options)` instead. Node calls do not print the CLI warning.
See the [migration guide](../migrations/deprecated-surfaces.md).

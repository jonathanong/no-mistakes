# `version-pin-consistency`

Keep a version pin in a structured source file in lockstep with the same
version restated in other tracked files.

```yaml
rules:
  - rule: version-pin-consistency
    scope: repository
    options:
      sourceFile: .mise.toml
      sourceKey: tools.aqua:example-org/example-tool
      anchors:
        - file: .github/actions/setup-example-tool/action.yml
          pattern: 'EXAMPLE_TOOL_VERSION:\s*(\d+\.\d+\.\d+)'
          label: example-tool
```

`sourceFile` is parsed as TOML when the path ends in `.toml`, JSON/JSONC when
it ends in `.json` or `.jsonc`, and YAML otherwise. Use the existing structured
parser for JSON/YAML; TOML is parsed with the `toml` crate.

`sourceKey` is a dotted path `section.key`. The key (everything after the
first `.`) may contain `:` and `/`, so ids such as
`tools.aqua:example-org/example-tool` resolve as `tools` then
`aqua:example-org/example-tool`. Nested maps are walked when the remainder is
itself dotted (`package.engines.node`). Configured paths are compared after
stripping a leading `./`.

Include and exclude globs apply: only `sourceFile` and anchors that remain in
the filtered tracked file list are checked or reported. The source pin may
still be read from disk to check remaining anchors. The rule is silent when
none of those files remain.

Each `pattern` must contain exactly one capturing group. The captured text
must equal the string pin at `sourceKey`. Missing keys, non-string pins,
missing captures, and mismatches are findings with file and line when
possible.

Counterexample: the source file pins `0.24.2` while an anchor still says
`0.24.1`.

```yaml
# .github/actions/setup-example-tool/action.yml
EXAMPLE_TOOL_VERSION: 0.24.1
```

Fix: update the source pin and every matching capture in the same change.

```yaml
EXAMPLE_TOOL_VERSION: 0.24.2
```

Use `no-mistakes-disable-next-line version-pin-consistency` or
`no-mistakes-disable-file` for a one-off exception.

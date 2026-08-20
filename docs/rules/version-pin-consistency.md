# `version-pin-consistency`

Keep a version pin in a structured source file in lockstep with the same
version restated in other tracked files.

```yaml
rules:
  - rule: version-pin-consistency
    scope: repository
    options:
      sourceFile: .mise.toml
      sourceKey: tools.aqua:lycheeverse/lychee
      anchors:
        - file: .github/actions/setup-lychee/action.yml
          pattern: 'LYCHEE_VERSION:\s*(\d+\.\d+\.\d+)'
          label: lychee
```

`sourceFile` is parsed as TOML when the path ends in `.toml`, JSON/JSONC when
it ends in `.json` or `.jsonc`, and YAML otherwise. Use the existing structured
parser for JSON/YAML; TOML is parsed with the `toml` crate.

`sourceKey` is a dotted path `section.key`. The key (everything after the
first `.`) may contain `:` and `/`, so ids such as
`tools.aqua:lycheeverse/lychee` resolve as `tools` then
`aqua:lycheeverse/lychee`. Nested maps are walked when the remainder is itself
dotted (`package.engines.node`).

Each `pattern` must contain exactly one capturing group. The captured text
must equal the string pin at `sourceKey`. Missing keys, non-string pins,
missing captures, and mismatches are findings with file and line when
possible.

The rule is silent when none of `sourceFile` or the anchor files are tracked.

Counterexample: the source file pins `0.24.2` while an anchor still says
`0.24.1`.

```yaml
# .github/actions/setup-lychee/action.yml
LYCHEE_VERSION: 0.24.1
```

Fix: update the source pin and every matching capture in the same change.

```yaml
LYCHEE_VERSION: 0.24.2
```

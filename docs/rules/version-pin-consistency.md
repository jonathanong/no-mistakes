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

When an anchor remains selected but its source is outside the selected file
set, the rule still reads the configured source from disk. If that source is
missing, unreadable, or malformed, the rule reports an unanalyzable finding at
line 1 of each selected anchor by default. Set `options.unanalyzable` to
`advise` to receive non-failing advice, or `ignore` to suppress this
uncertainty. Restore the source file or correct its path/format to let the rule
compare the anchor. A valid source outside the selected set still supplies the
comparison pin; a definite anchor mismatch remains a failure under every
unanalyzable policy.

If a selected anchor itself cannot be read as UTF-8, the rule reports the read
error at line 1 of that anchor through the same `unanalyzable` policy. Restore
the anchor as readable UTF-8 text to resume the comparison.

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

Use `no-mistakes-disable-file version-pin-consistency` in the anchor file
for a file-level exception. Both findings and advisories honor it.

## Why and when

Use this rule when one structured source owns a tool version and workflows,
actions, or docs repeat it as operational anchors.
If a selected anchor's source is unavailable outside the selected file set, the
rule cannot verify the pin and reports or advises at that anchor instead of
silently dropping it.

## What it catches/requires

The source key must resolve to a string pin and every included anchor's single
capture group must equal it. Missing keys, stale captures, malformed patterns,
and stale paths are findings. If a selected anchor depends on a missing,
unreadable, or malformed source that is outside the selected file set, that
uncertainty follows `options.unanalyzable` (`report` by default, `advise`, or
`ignore`). A selected anchor that cannot be read as UTF-8 follows the same
policy. A valid source outside the selected file set still supplies the
comparison pin, and definite mismatches always fail.

## Options and defaults

`sourceFile`, `sourceKey`, and `anchors` are required. Each anchor supplies a
file and exactly-one-capture `pattern`, with optional `label`. Common
include/exclude filters apply; there are no implicit source or anchor files.

`options.unanalyzable` inherits the repository policy, which defaults to
`report`. It accepts `report`, `advise`, or `ignore` for incomplete comparisons
caused by an unreadable or malformed source outside the selected file set or
an unreadable selected anchor.
Definite mismatches and validation of a selected source remain findings.

## Valid example

```toml
# .mise.toml
[tools]
"aqua:example-org/example-tool" = "0.24.2"
```

```yaml
# .github/actions/setup-example-tool/action.yml
EXAMPLE_TOOL_VERSION: 0.24.2
```

## Counterexample

The structured source says `0.24.2` while an anchor captures `0.24.1`.

```toml
# .mise.toml
[tools]
"aqua:example-org/example-tool" = "0.24.2"
```

```yaml
# .github/actions/setup-example-tool/action.yml
EXAMPLE_TOOL_VERSION: 0.24.1
```


## Fix

Update every anchor in the same change, or change the source ownership and
patterns together so one canonical pin remains. For an unanalyzable finding,
restore the source file or correct `sourceFile`/its format, or restore an
unreadable anchor as UTF-8 text, so the selected anchor can be compared.

## Analysis

### Effort

Runs at `max` only. Configuring `effort: low` is accepted: `check` emits an analysis warning (requested effort low; effective effort max because this rule has no low implementation) and runs the rule at `max`.

### Unanalyzable inputs

Following recall over precision, inputs this rule cannot prove are never treated as clean ([how analysis works](../../README.md#how-analysis-works)); they follow `options.unanalyzable`:

- A selected anchor whose `sourceFile` is outside the selected file set and is missing, unreadable, or malformed.
- A selected anchor that cannot be read as UTF-8.

Policies:

- `report` (default): fails as an `unanalyzable` finding at line 1 of the anchor.
- `advise`: a non-failing advisory.
- `ignore`: omitted.

Definite anchor mismatches remain findings under every policy.

```yaml
rules:
  - rule: version-pin-consistency
    scope: repository
    options:
      unanalyzable: advise
      sourceFile: .mise.toml
      sourceKey: tools.aqua:example-org/example-tool
      anchors:
        - file: .github/actions/setup-example-tool/action.yml
          pattern: 'EXAMPLE_TOOL_VERSION:\s*(\d+\.\d+\.\d+)'
```

## Suppression

Use `no-mistakes-disable-file version-pin-consistency` in the anchor file for
an exception that covers findings and advisories when the anchor is readable.
Incomplete-comparison diagnostics point to line 1, so
`no-mistakes-disable-line` must be on that line. Definite mismatches retain the
captured version's line and support `no-mistakes-disable-next-line` immediately
before that line.

## Related rules

[`test-no-dependency-pins`](test-no-dependency-pins.md) prevents tests from
hard-coding these versions; [`pnpm-release-age-policy`](pnpm-release-age-policy.md)
governs release-age exceptions.

# `github-actions-pinned-hash`

## Why and when

Use this rule when CI action code must be reproducible and a reviewer must be
able to identify the version represented by each immutable SHA.

## What it catches

It rejects mutable tag or branch refs, non-40-character refs, and SHA refs
without a version-shaped trailing comment. Local and Docker actions are exempt.
Flow mappings, quoted `uses` keys, values on the line after `uses:`, including
a flow mapping that breaks after the key, and YAML aliases are checked the
same way. An alias is judged as the action reference it resolves to. Each YAML
document is walked, so a flow `uses` in a later document is still checked. A
quoted flow scalar folded across lines is reported on the `uses` key line. A
`uses` key nested under `with` or `env` is an action input, not an action
reference, including when that flow mapping continues on the next line. Every
`uses` on one physical line is checked. A `|` or `>` script is not an action
reference; the following line of a block `uses` is. A `uses` field under
`strategy.matrix`, including a flow `include` entry, is not an action reference.

## Options

`excludePaths` is the only rule-specific option and defaults to an empty list.
The built-in workflow and composite-action locations remain the default scope.

Requires every `uses:` step in GitHub Actions workflow files to reference a
40-char commit SHA with a version comment. Tag- or branch-pinned actions are
mutable — an upstream owner can move the ref at any time, silently swapping
the action's code.

```yaml
rules:
  - rule: github-actions-pinned-hash
    scope: repository
```

## Valid example

The two `uses:` lines below are valid because each has a full SHA and a
version comment.

```yaml
- uses: actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd # v6.0.2
- uses: dtolnay/rust-toolchain@e2a55d2ffb04f378e9626c28d38b36d230d1e12f # 2026.10.09
  with:
    toolchain: stable
```

## Counterexample

```yaml
- uses: actions/checkout@v6.0.2         # tag — fails
- uses: dtolnay/rust-toolchain@stable   # branch — fails
- uses: actions/checkout@de0fac2e45...  # SHA but no version comment — fails
- uses: actions/checkout@de0fac2e45...  # main  # comment not a version — fails
```

## Fix

Pin to the commit SHA for the desired tag and record the version in a
trailing comment:

```bash
gh api repos/actions/checkout/commits/v6.0.2 --jq '.sha'
```

Then update the workflow line:

```yaml
- uses: actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd # v6.0.2
```

The comment must start with a version number (`# v1.2.3`, `# v2`, or
`# 1.96.0`). For an untagged action, a numeric snapshot date such as
`# 2026.10.09` also passes. The Rust example pins the setup action snapshot
while selecting the latest stable compiler through its input. Branch-name and
tool-name comments (`# main`, `# stable`) are rejected.

**Exemptions:** Local action references (`./`, `../`) and Docker image
references (`docker://`) are never checked. Use `excludePaths` to opt out
specific files:

```yaml
rules:
  - rule: github-actions-pinned-hash
    scope: repository
    options:
      excludePaths:
        - .github/workflows/release.yml
```

## Analysis

### Effort

- Runs at `max` only. Configuring `effort: low` (on the rule, or as a repository or CLI default) is not an error: `check` records the warning `requested effort low; effective effort max because this rule has no low implementation` and runs the rule at `max`.

### Unanalyzable inputs

Upgrade note: in-scope files that cannot be read, or parsed where this rule
requires parsing, now fail `check` under the default `unanalyzable: report`
instead of silently passing. Fix or exclude those inputs, or set
`options.unanalyzable: advise` or `ignore` to restore the previous successful
exit status for uncertainty alone. Independent definite findings still fail
under every policy.

Selected files that cannot be read as UTF-8 produce an `unanalyzable`
diagnostic instead of silently passing. Restore read access or repair the
encoding so the rule can verify the file. The shared `options.unanalyzable`
setting inherits the repository policy: `report` (the default) fails,
`advise` emits non-failing advice, and `ignore` omits uncertainty. Definite
violations remain failures under every policy, including when `effort: low`
falls back to `max`.

Configured scope, includes, excludes, and rule-specific exemptions still
select the inputs. Suppressions apply to both channels, including directives
retained in an invalid UTF-8 read error; source bytes are never reinterpreted
as analyzable code. Rule-level custom messages apply to these diagnostics.

YAML parse failures also use this policy. Definite pin violations recovered
by the line scan remain findings when YAML cannot be parsed, including with
`advise` or `ignore`.

## Suppression

Use `no-mistakes-disable-next-line github-actions-pinned-hash` to suppress a
single line.

**Scope:** Checks `.github/workflows/*.{yml,yaml}` and
`.github/actions/**/action.{yml,yaml}` composite action files.

## Related rules

[`github-actions-action-timeout-pair`](github-actions-action-timeout-pair.md)
and [`github-actions-job-timeouts`](github-actions-job-timeouts.md) enforce
the other per-step and per-job workflow contracts.

# `required-local-docs`

Requires documentation beside configured code directories.

```yaml
rules:
  - rule: required-local-docs
    scope: repository
    options:
      roots: [agents]
      requiredFile: README.md
      codeExtensions: [mts, ts]
```

Counterexample: `agents/email/index.mts` without `agents/email/README.md`.

Fix: add the local doc or adjust roots/excludes.

## Why and when

Use this rule when each code directory needs a nearby README explaining its
ownership, entrypoint, or agent workflow.

## What it catches/requires

Each first-level directory under a root that contains a matching code file
anywhere beneath it must contain `requiredFile` directly in that directory.
Nested directories do not each need their own copy. `codeExtensions`
determines which files make a directory relevant. Roots are
normalized before the scan: `./`, empty segments, and in-repo `..` collapse,
so `services/../agents` is `agents`. A root that leaves the repository, such
as `..`, is one finding on the config file at line 1
(`..: configured root escapes the repository`). The parent directory is not
scanned.

## Options and defaults

`roots` is required; an empty list disables the rule. `requiredFile` defaults
to `README.md`; `codeExtensions` defaults to `ts`, `mts`, `cts`, `js`, `jsx`,
`tsx`, `sql`, and `rs`; and `testExcludePatterns` defaults to `*.test.*`,
`*.spec.*`, and `__tests__`. Common excludes still apply.

`glob` and `requiredHeading` are not options for this rule: they belong only to
the sibling [`required-doc-section`](required-doc-section.md) rule.

## Valid example

```text
agents/email/index.mts
agents/email/README.md
```

## Counterexample

```text
agents/email/index.mts
```

## Fix

Add the required local README, or remove the directory from the configured roots
if it is generated or owned by another documentation system.

## Analysis

### Effort

Runs at `max` only. Configuring `effort: low` is accepted: `check` emits an analysis warning (requested effort low; effective effort max because this rule has no low implementation) and runs the rule at `max`.

### Unanalyzable inputs

Following recall over precision, nothing is silently treated as clean ([how analysis works](../../README.md#how-analysis-works)); this rule has no unanalyzable inputs because it checks file presence in the file inventory, so `options.unanalyzable` has no effect on it.

## Suppression

Missing-document findings point to the directory. Adjust `roots` or common
excludes for a generated directory that should be exempt. The finding for a
configured root outside the repository points to line 1 of the configuration
file. A leading `no-mistakes-disable-file required-local-docs` directive in that
file suppresses the finding.

## Related rules

[`required-doc-section`](required-doc-section.md) enforces headings inside
existing Markdown; [`require-files-in-subdirs`](require-files-in-subdirs.md)
can require several files per directory.

# `markdown-structure-budget`

Limits visual density in large Markdown files. A file is oversized when it has
more than `maxLines` physical lines or more than `maxChars` Unicode scalar
characters; it then allows at most `maxTables` GFM tables and `maxMermaid`
fenced Mermaid blocks.

## Why and when

Use this rule when large Markdown files need visual structure without becoming
an unreadable wall of tables and diagrams.

## What it catches

It reports selected Markdown that exceeds either size threshold and then exceeds
the table or Mermaid budget, as well as stale or malformed baseline entries.

## Options

`maxLines`, `maxChars`, `maxTables`, and `maxMermaid` default to `180`,
`12000`, `1`, and `1`. `baselineFile` is optional and must be a tracked JSON
object with the documented table/Mermaid counts.

The comparison is strict: exactly 180 lines or 12,000 characters does not
trigger the default budget. Mermaid is identified from the first fenced-code
info token case-insensitively; indented code is not a Mermaid block.

Finding paths are lexical from the request root. External configured projects
use a stable `../project/file.md` path, so standard file suppressions resolve
back to the source. Baseline entries remain relative to each effective project
root, keeping a project's rollout state portable across request locations.

For staged rollout, `baselineFile` may name a tracked JSON object mapping each
current violation to exact `{ "tables": N, "mermaid": N }` counts. Any count
change, resolved/deleted path, malformed JSON, or new violation fails.

## Valid example

A document below both size thresholds, or an oversized document at or below
both visual budgets, passes without a baseline entry.

The `maxLines: 5` below is for illustration only so the tiny Counterexample is
oversized; the documented default remains 180.

```yaml
rules:
  - rule: markdown-structure-budget
    scope: repository
    include: ["**/*.md"]
    options:
      maxLines: 5
      maxChars: 12000
      maxTables: 1
      maxMermaid: 1
```

```md
# Title

One short paragraph stays under the line budget.
```

## Counterexample

A document over `maxLines` that also contains two GFM tables exceeds the
one-table budget.

```md
# Title

| a | b |
| --- | --- |
| 1 | 2 |

| c | d |
| --- | --- |
| 3 | 4 |
```

## Fix

Split the document into focused README-indexed documents. Do not replace a
useful table or diagram with less-readable syntax merely to evade this rule.

## Analysis

### Effort

Runs at `max` only. Configuring `effort: low` is accepted: `check` emits an analysis warning (requested effort low; effective effort max because this rule has no low implementation) and runs the rule at `max`.

### Unanalyzable inputs

Following recall over precision, nothing is silently treated as clean ([how analysis works](README.md#how-analysis-works)); this rule has no unanalyzable inputs because it counts structural elements in parsed Markdown, so `options.unanalyzable` has no effect on it.

## Suppression

Suppressions use standard `no-mistakes` directives and should be exceptional.

## Related rules

[`markdown-mermaid-validation`](markdown-mermaid-validation.md) validates each
diagram; [`agents-md-max-size`](agents-md-max-size.md) is the instruction-file
size policy.

# `markdown-structure-budget`

Limits visual density in large Markdown files. A file is oversized when it has
more than 180 physical lines or more than 12,000 Unicode scalar characters;
defaults then allow at most one GFM table and one fenced Mermaid block.

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

## Valid example

A document below both size thresholds, or an oversized document at or below
both visual budgets, passes without a baseline entry.

```yaml
rules:
  - rule: markdown-structure-budget
    scope: repository
    include: ["**/*.md"]
    options:
      maxLines: 180
      maxChars: 12000
      maxTables: 1
      maxMermaid: 1
```

Counterexample: an 181-line document containing two GFM tables. It exceeds the
line threshold and exceeds the default one-table budget.

The comparison is strict: exactly 180 lines or 12,000 characters does not
trigger the budget. Mermaid is identified from the first fenced-code info token
case-insensitively; indented code is not a Mermaid block.

Finding paths are lexical from the request root. External configured projects
use a stable `../project/file.md` path, so standard file suppressions resolve
back to the source. Baseline entries remain relative to each effective project
root, keeping a project's rollout state portable across request locations.

For staged rollout, `baselineFile` may name a tracked JSON object mapping each
current violation to exact `{ "tables": N, "mermaid": N }` counts. Any count
change, resolved/deleted path, malformed JSON, or new violation fails.

Fix: split the document into focused README-indexed documents. Do not replace a
useful table or diagram with less-readable syntax merely to evade this rule.

Suppressions use standard `no-mistakes` directives and should be exceptional.

## Related rules

[`markdown-mermaid-validation`](markdown-mermaid-validation.md) validates each
diagram; [`agents-md-max-size`](agents-md-max-size.md) is the instruction-file
size policy.

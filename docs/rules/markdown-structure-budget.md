# `markdown-structure-budget`

Limits visual density in large Markdown files. A file is oversized when it has
more than 180 physical lines or more than 12,000 Unicode scalar characters;
defaults then allow at most one GFM table and one fenced Mermaid block.

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

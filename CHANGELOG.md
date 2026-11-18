# Changelog

## Unreleased

- Fix `postgres-grouping-not-aggregation` per-rule `options.unanalyzableSql: report`
  to accept it as an alias for non-failing `advise`. Canonical
  `options.unanalyzable` takes precedence; invalid alias values remain errors.
- Compatibility reminder for output introduced by #3554, predating this patch:
  the grouping rule's default `undeclared-executor-type` residual advice includes
  an application index in its message, for example
  `postgres-grouping-not-aggregation application #0 has unresolved SQL for undeclared-executor-type:`,
  and omits the former `import` field. Consumers should tolerate the missing
  field and select records by `rule: postgres-grouping-not-aggregation` and
  `target: undeclared-executor-type` rather than message text. This advice remains
  non-failing. The report-alias fix does not introduce this output format.

# Check SQL verdicts

`check(options)` returns the same structured check report as CLI JSON,
including configured filesystem rules such as
`package-json-nested-workspace-coverage`, and `warnings: string[]` for checks
that could not run. `check()` and `checkJson` also return optional
`unprovenMemberQuerySql` (`number` on the exported `CheckReport`) when
SQL-looking member `.query` calls were not proven to be executors. The field
is omitted when the count is zero. It is not part of `warnings` and does not
set a failure. Those APIs return the count and do not print the CLI stderr
line. An `analyzeProject()` report of type `check` includes the same field.


`check({ includeSqlVerdicts: true })` adds a diagnostic `sqlVerdicts` array to
the asynchronous check report. A `type: "check"` report in `analyzeProject()`
accepts the same option. Omitting it preserves the existing report fields and
skips rule-level proof recording unless a rule requests residual advice.

Each exported `SqlObligationVerdict` identifies a configured executor profile,
source file, half-open call byte span, rule, `SqlRuleApplication` ordinal/name,
enabled subcheck, proof status,
evidence kind when proven, and residual uncertainty reasons. The exported
`SqlVerdictStatus` union is `"violation" | "proven-clear" | "abstain"`;
`SqlEvidenceKind` names exact SQL, an exhaustive union, a verified prefix, or
an explicit policy exemption.
`knownPrefix` and `helperLines` appear only when prepared analysis has that
source evidence. See [SQL verdict semantics](../postgres-sql-verdicts.md).

Rule application `unanalyzableSql: advise` emits non-failing residuals in the
existing `advisories` field without requiring `includeSqlVerdicts`. Proven
findings retain their usual `rules` field and suppression behavior. The
`SqlObligationVerdict` array itself appears only on request.
It is a raw proof inventory: suppressed calls remain in `sqlVerdicts` while
their findings and advisories follow normal suppression directives.

Grouped residual advice, including a `target` that may list several checks, is
described in [SQL verdict semantics](../postgres-sql-verdicts.md).

Pass `effort: "low"` or `effort: "max"` to `check()` or a check report in
`analyzeProject()`. The default is `max`; invocation effort takes precedence
over per-rule and repository defaults. If a low-effort rule's required partner
check is missing or out of scope, the report includes a warning and that rule
runs at `max`. Rule options may set `effort` and the shared `unanalyzable`
policy. `unanalyzableSql` is a backward-compatible SQL alias for the same
policy. Legacy PostgreSQL rules that use `fail` keep that spelling; shared
`report` maps to the legacy parser's accepted value.

[Node/N-API guide](../node-api.md)

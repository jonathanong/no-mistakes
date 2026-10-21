# Declared runner config deadlines

`check({ includeRunnerConfigDeadlines: true })` exposes the CLI's optional `runnerConfigDeadlines` evidence. `analyzeProject({ reports: [{ type: "check", includeRunnerConfigDeadlines: true }] })` uses the same prepared owner and renderer. Mixed check reports prepare their union demand once; an ordinary sibling report does not receive the optional field. Existing asynchronous bindings/facades remain unchanged.

The exported `RunnerConfigDeadlineEvidence`, `ConfigDeadlineEvidence`, `ProjectDeadlineEvidence`, `DeclaredDeadlineSlot`, `DeclaredDeadlineUnknownReason`, `DeadlineProvenance` and `DeadlineInheritanceEvidence` describe the tagged statuses, values and provenance. Slots are absent, known (including zero/negative/over-cap literals), or unknown with a typed reason. Paths are request-root-relative where possible; spans are byte offsets or null. Evidence survives suppression and empty findings, while ordinary integration errors are still errors. No SDK default or complete registration/runtime closure is inferred. See [CLI contract](../cli/check.md#declared-runner-config-evidence).

`parsePostgresSql()` SELECT facts expose typed scalar array `ANY`/`ALL` comparisons
through `query.quantifiedComparisons`; operand coverage contributes to query and
INSERT source completeness. See [PostgreSQL source facts](../postgres-source-api.md).

<!-- cspell:ignore THREADPOOL -->

[Node/N-API guide](../node-api.md)

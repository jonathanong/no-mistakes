# Change classes

Every pull request belongs to one class. Reviewers state the class at the top
of a review, with the `file:line` that puts a change in the breaking or
architectural class. The class decides who may merge it.

| Class | Who merges |
| --- | --- |
| Breaking | A maintainer, after discussion |
| Architectural | A maintainer, after discussion |
| Maintenance | An orchestrator with merge authority, after review |
| None (an ordinary fix or additive feature) | An orchestrator with merge authority, after review and consumer validation |

## Breaking

A change that can break an existing caller or change documented output without
the caller changing its own configuration:

- Removing or renaming a CLI command, flag, or JSON report field.
- Removing or renaming an N-API export or an exported `.d.ts` type.
- Changing an exit code.
- New findings from a rule that is on by default.
- Broadening an opt-in rule's default behavior to reject previously documented
  valid patterns without a new option.

Additive options, new opt-in rules, and new report fields are not breaking, as
long as existing fields keep their meaning (see
[Prepared analysis ownership](../../AGENTS.md#prepared-analysis-ownership)). A
correctness fix that adds true-positive findings to an opt-in rule — even for
repositories that already configure that rule — is an ordinary fix (`None`), not
breaking, provided it stays within the rule's documented scope and keeps options
and report fields unchanged. Consumer validation still applies before merge.

## Architectural

A change to how the analyzer is built rather than what it reports:

- A new dependency on a remote service, another repository, or version-control
  history.
- Persisted state or caches (see [why](../why.md)).
- A new shape for the shared analysis session, canonical graph, or resolver.
- A new top-level command family.

## Maintenance

CI-only, test-only, docs-only, or dependency-only changes with no change to
analyzer behavior.

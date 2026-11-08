# Suppression hygiene

`check` reports `no-mistakes-disable-line`,
`no-mistakes-disable-next-line`, and `no-mistakes-disable-file` comments that
suppressed no finding during the run. This catches stale exceptions when code
or a finding's location changes. Each diagnostic names the rule and points to
the comment's file and line.

```yaml
# Repository setting; not a rule's options.
reportUnusedDisableDirectives: advise
```

| Value | Behavior |
| --- | --- |
| `advise` (default) | Return non-failing diagnostics in `advisories`. |
| `report` | Return diagnostics in `rules`; CLI `check` exits nonzero. |
| `ignore` | Omit suppression hygiene diagnostics. |

`unused-disable-directive` means the named check rule is enabled but the
comment hid nothing. `invalid-disable-directive` separately identifies an
unknown rule name, a known rule that is disabled, or an ineffective comment
placement. Use the canonical rule ID,
not a rule application's `name`. The structured finding's `target` contains
that ID; its `rule` identifies the hygiene diagnostic.

For example, with `postgres-explicit-columns` enabled for this SQL file:

```sql
-- no-mistakes-disable-next-line postgres-explicit-columns: legacy projection
SELECT * FROM users;
-- no-mistakes-disable-next-line postgres-explicit-columns: stale exception
SELECT id FROM users;
```

The first comment is used. The second produces `unused-disable-directive` at
the comment line. Remove it. If a justified exception stopped matching after
a finding moved, place it at the reported statement instead and rerun the
check. Fix a misspelled rule ID or remove a comment for a disabled rule.

The audit scans the union of declared rule applications' project or repository
roots, narrowed by their top-level `include` and `exclude`. It includes disabled
applications so their stale exceptions remain visible. Rule-specific input
options are not reinterpreted as audit file filters. This means a directive in
a file skipped by a rule's own `exclude` option (or an option such as
`sqlInclude`) can still be reported as unused: the audit selects files from the
declared roots and top-level filters, not from that rule's input options. A
known rule outside its enabled application scope is reported as disabled or out
of scope.

PostgreSQL statement rules also audit next-line comments inside statically
recovered executor SQL. For example, after an indexed LIMIT-bounded probe is
accepted by `postgres-required-predicates`, remove its in-template
`-- no-mistakes-disable-next-line postgres-required-predicates` comment.
The audit reuses prepared executor projections and physical source mappings;
unexecuted SQL strings are not added to the audit. Only directives that the
existing statement suppression matcher can consume are included.

Each directive names one rule ID. Put separate directives on separate lines
when a file needs exceptions for multiple rules:

```ts
// no-mistakes-disable-file rule-a: legacy exception
// no-mistakes-disable-file rule-b: same legacy exception
doSomething();
```

Comma-separated rule IDs are not supported. For example,
`no-mistakes-disable-next-line rule-a, rule-b` does not match `rule-a`: the
comma is not a valid separator between the rule ID and an optional reason.
`rule-b` is not matched either. The `no-mistakes-disable-file` and
`no-mistakes-disable-line` forms follow the same one-rule grammar.

The result describes this invocation. It does not establish whether an
exception would be needed under a different configuration.

Markdown and MDX code fences, indented code, and inline code are examples rather
than live comments. Only HTML comments outside those code regions participate
in their audit. Rust strings and raw strings likewise do not create audit
directives. Ordinary Rust block comments (`/* ... */`) are ignored, including
their contents; genuine supported line comments still do.

Rust documentation comments (`///`, `//!`, `/** ... */`, and `/*! ... */`)
are documentation, not suppression comments. A directive written in one is
reported as `invalid-disable-directive` because its placement is ineffective,
even when the named rule ID is valid and enabled. The finding's `target` is the
named rule ID and its `rule` is `invalid-disable-directive`. Move a real
exception to a plain `//` comment at the code location where it should apply:

This applies to both line and block documentation forms, whether they document
an item (`///`, `/** ... */`) or a module (`//!`, `/*! ... */`). For example,
each directive below is documentation and cannot suppress the rule:

```rust
//! no-mistakes-disable-next-line some-rule: module docs, ineffective
/*! no-mistakes-disable-next-line some-rule: module docs, ineffective */

/// no-mistakes-disable-next-line some-rule: documentation, ineffective
fn line_docs() {}
```

```rust
/// no-mistakes-disable-next-line some-rule: this does not suppress the check
fn example() {}

// no-mistakes-disable-next-line some-rule: justified exception
fn legacy_example() {}
```

For block documentation comments, the same distinction applies. A directive
outside a fenced example is invalid, but an ordinary block comment containing
the same text is ignored:

```rust
/** no-mistakes-disable-next-line some-rule: ineffective documentation directive */
fn documented_api() {}

/* no-mistakes-disable-next-line some-rule: ordinary comment, ignored */
fn ordinary_comment() {}
```

Fenced examples inside Rust documentation comments remain documentation and
are ignored by the audit. This lets docs show directive syntax without creating
an ineffective-directive finding:

````rust
/**
 * Example syntax:
 * ```rust
 * // no-mistakes-disable-next-line some-rule: example only
 * fn example() {}
 * ```
 */
fn documented_block_api() {}
````

Exclude test data with the rule application's top-level `exclude`, for example
`exclude: ["fixtures/**", "test-cases/**"]`. Apply those exclusions to every
rule application whose declared scope includes the test data: the audit uses
the union of their scopes. Rule-specific input options do not exclude files
from that union.

These diagnostics are added after suppression filtering; another disable
comment cannot hide them. Configure `ignore` to turn the audit off.

The audit reuses the request's source store and retains comment metadata before
analysis. Checks that otherwise need only paths can therefore read additional
selected source text. `ignore` skips that audit work.

CLI `check`, async `check()`, and `analyzeProject()` reports with
`type: "check"` use the same repository setting and diagnostic channels.
`includeSuppressed` remains an independent, opt-in ledger of findings actually
hidden by directives; it is not required to report unused comments. A React
component can have one public ledger row while several fetch directives count
as used, including when an unsuppressed fetch keeps the component's violation
visible. An entry-file directive on a Next.js config also counts when it hides
a finding attributed to a delegated helper.

This audit covers no-mistakes check rules. ESLint uses its own
`reportUnusedDisableDirectives` setting.

## Allow-entry hygiene

Configured exception lists share invocation-local usage accounting. An entry
counts as used when its selector matches a finding before allow filtering.
All overlapping entries matching that finding count as used; one entry does
not become stale merely because an earlier entry also matches it.

Stale diagnostics use `rule: "unused-allow-entry"`, with the
configuration file and entry line in `file` and `line`, the checked rule ID in
`import`, and the selector in `target`. Entry lines refer to the authored
sequence item under that rule option, even when an earlier rule has already
been checked or contains the same string. Their message names the rule and option.
Remove an obsolete entry, or correct its selector if the exception is still
needed. Reporting unused exceptions prevents obsolete intent from silently
hiding future violations.

Set the repository policy independently of disable-comment hygiene:

```yaml
reportUnusedAllowEntries: advise
```

| Value | Behavior |
| --- | --- |
| `advise` (default) | Return `unused-allow-entry` diagnostics as non-failing `advisories`. |
| `report` | Return them in `rules`; CLI `check` exits nonzero. |
| `ignore` | Omit `unused-allow-entry` diagnostics. |

Changing the policy retains the same diagnostic message, file, line, rule,
import, and target. `reportUnusedDisableDirectives` configures comments only.
Disable comments and a rule application's advisory severity cannot demote or
suppress these hygiene diagnostics. CLI `check`, async `check()`, and
`analyzeProject()` check reports use the same policy and report shape.
Established rule-specific stale findings described below keep their existing
contracts; this setting controls only `unused-allow-entry` diagnostics.

An entry is stale only when the run checked its entire relevant selector
scope and none of those checks produced a matching finding. For a file glob,
every matching file in the request's visible inventory must have been checked
successfully by that rule application. A partially checked glob stays silent,
even when no finding matched the checked subset. Excluded files, changed-file
restrictions, disabled applications, missing files, read/parse failures, and
uncertain coverage cannot establish staleness. An empty match set is not proof
of a checked scope. Graph-dependent checks also require usable supporting facts.
The audit never widens a run or parses files again to establish coverage.
Regex and package-name entries have no file selector, so the audit requires
complete coverage of their visible file domain; otherwise it stays silent.

For example, `allow: ["src/**"]` cannot be reported stale when the run checked
only `src/one.ts` and left `src/two.ts` outside scope. It can be reported when
all matching relevant files were checked and none needed the exception.

New advisory reporting covers `query-reached-per-item`, `no-raw-ephemeral-port`,
`swift-no-raw-print`, `swift-viewmodel-main-actor`,
`csharp-no-async-void-delegate`, `file-extension-policy`, `lockfile-allowlist`,
`package-json-workspace-coverage`, `workspace-package-cycles`,
`postgres-json-relation-ids`, nested `forbidden-dependencies` allowed files,
`no-test-git-sha` allowed contexts, `test-email-domain-policy` allowed email
patterns, `package-json-required-fields` unscoped-name exceptions, and
`no-git-identity-mutation` conditional workflow allowances. Schema-backed
JSON-relation allowances require complete applicable catalog coverage;
combined catalog/write checks remain silent when coverage is uncertain.

For `query-reached-per-item`, prefer an allowance such as
`{file: src/tasks.ts, function: backfillPage, callee: query}`.
The optional `function` selects the exact enclosing caller name shown in the
finding's call path; `callee` and one-based `line` further narrow it. All supplied
selectors must match the same finding. A finding in another function cannot
mark that entry used. Existing file-only allowances still suppress the whole
file and are still reported when unused under `reportUnusedAllowEntries`.

Existing PostgreSQL catalog allow lists use the shared usage accounting while
keeping their established stale messages, locations, and severity. Existing
workflow-timeout, Markdown-eval, Storybook, tsconfig, and SQL allow-list
reporters retain their established diagnostic contracts, including their
existence-based checks where applicable. They no longer call an entry stale
solely because its files were outside the checked scope. Existing in-scope
stale findings can therefore still fail `check`; the new advisory diagnostic
does not demote them or duplicate their reports.

Policy vocabularies such as allowed SQL types, accepted YAML keys, and permitted
manifest fields define valid input rather than individual finding exceptions.
They are not audited as suppressions. File and root exclusions continue to
control analysis scope rather than becoming allow entries.

### Coverage boundaries

The lists above describe implemented hygiene coverage, not every option whose
name contains `allow`, `ignore`, or `exempt`. These additional exception selectors
use the same finding-based accounting:

| Rule | Option | Complete-scope requirement |
| --- | --- | --- |
| `no-empty-or-comments-only-files` | `intentionallyEmpty` | The exact selected file was read successfully with a supported extension. Only empty or comment-only content uses the exception; missing, excluded, unreadable, or unsupported files cannot prove staleness. |
| `postgres-required-comments` | `exemptColumnNamePatterns` | Column checks are enabled and the applicable catalog is complete. Only otherwise reportable missing or short comments use each overlapping pattern. |
| `postgres-column-naming` | `ignoreTablePatterns` | The applicable catalog is complete. Only otherwise reportable column-policy findings use each overlapping table pattern. |
| `postgres-finite-text-columns` | `ignoreTablePatterns` | The applicable catalog is complete. Only otherwise reportable finite-text findings use each overlapping table pattern. |
| `postgres-object-naming` | `plural.ignorePatterns` | Plural checks are enabled and the applicable catalog is complete. Only otherwise reportable pluralization findings use each overlapping pattern. |
| `postgres-sql-shape-policy` | `shapeOptions.keysetOnlySweep.ignoreTables` | The configured SQL and embedded-executor domain is fully checked. Only otherwise reportable full-table sweeps use matching selectors; partial, failed, or unrecoverable analysis cannot prove staleness. |

For example, a table ignore pattern matching only compliant columns is stale;
matching a table name alone does not use the exception. Every overlapping
entry that would suppress a finding counts as used. The hygiene setting does
not widen the analysis scope or add another source/parse pass.

Several other exception lists already have their own stale diagnostics:
`postgres-no-add-column.allowedMigrations`,
`postgres-fk-index.allowedColumns` and `allowedTables`,
`postgres-redundant-index.allowedIndexes`,
`postgres-required-predicates.partitionKeyExemptions`, and
`postgres-column-naming.foreignKeys.requireForeignKey.exempt`. These retain
their rule IDs, messages, locations, severity, and existing matching semantics;
they do not also emit `unused-allow-entry`. Workflow topology's
`unlockedWorkflowReasons` likewise has its own stale-reason check.

Policy vocabularies are deliberately outside this audit. This includes
`strict-package-layout.allowedRootFiles` and `allowedSubdirs`,
`production-dependency-declarations.allowedFields`,
`github-actions-composite-step-schema.allowedKeys`, PostgreSQL accepted column
or array element types, `postgres-explicit-columns.allowWholeRowFunctions`,
`postgres-object-naming.doubleUnderscore.allowPattern`,
`postgres-sql-shape-policy.shapeOptions.literalLimit.allowedValues`, and
`workflow-topology-policy.secretInventory.builtInExemptions`. They define
valid layouts, fields, types, function forms, values, or platform-provided
secret names, rather than individual finding exceptions. Secret inventory's
`reportUnused` audits declared repository and environment secrets under its
own contract; it does not audit built-in exemptions. Boolean policy switches,
root/file exclusions, and include filters also remain outside allow-entry
accounting.

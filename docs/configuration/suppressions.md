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

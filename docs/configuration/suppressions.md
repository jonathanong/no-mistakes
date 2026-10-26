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
unknown rule name or a known rule that is disabled. Use the canonical rule ID,
not a rule application's `name`.

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

The result describes this invocation's configured check scope. It does not
establish whether an exception would be needed under a different configuration.
These diagnostics are added after suppression filtering; another disable
comment cannot hide them. Configure `ignore` to turn the audit off.

CLI `check`, async `check()`, and `analyzeProject()` reports with
`type: "check"` use the same repository setting and diagnostic channels.
`includeSuppressed` remains an independent, opt-in ledger of findings actually
hidden by directives; it is not required to report unused comments. This audit
covers no-mistakes check rules. ESLint uses its own
`reportUnusedDisableDirectives` setting.

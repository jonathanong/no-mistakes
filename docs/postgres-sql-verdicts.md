# PostgreSQL SQL verdicts

`no-mistakes check --include-sql-verdicts --format json` adds `sqlVerdicts` to
the check report. `check({ includeSqlVerdicts: true })` and a check report in
`analyzeProject()` expose the same array. The field is absent by default.

Each row identifies a configured executor profile, rule application, enabled subcheck,
source file, half-open byte span of the call, and one status:

`application.index` is the zero-based ordinal among enabled applications of
that rule in configuration order; `application.name` appears when configured.
Two overlapping applications remain separate rows even when they inspect the
same call and reach different verdicts.

- `violation`: a policy violation is proven for at least one complete SQL
  alternative, or a verified prefix proves it independently of an opaque tail.
- `proven-clear`: every exhaustive alternative satisfies that subcheck, or a
  verified prefix settles a prefix-only obligation such as a leading annotation.
- `abstain`: missing SQL, incomplete alternatives, parse failure, or catalog
  uncertainty prevents either conclusion.

`evidence` is `exact`, `exhaustive-union`, `verified-prefix`, or
`policy-exemption` when a proof exists. A safe directive can prove the policy
exempt without claiming the underlying SQL is complete. `residualReasons`
names remaining uncertainty. `knownPrefix` and
`helperLines` appear when prepared analysis has that source evidence. A finite
`Possible` value with an implicit unknown alternative is not an exhaustive
union. Overflow beyond the bounded alternative cap also abstains. Standalone
SQL-builder fragments are distinct from configured executor calls.
The optional array is a raw proof inventory, including calls whose diagnostic
finding or advisory is suppressed. Suppression controls the `rules` and
`advisories` lists; it does not erase the call or alter its proof status.
An existing contextual finding can coexist with whole-call `abstain` when
another reachable invocation remains unknown; the row does not promote one
observed context into proof for every execution.

Each PostgreSQL rule application accepts `unanalyzableSql: advise` to place its
residual uncertainty in the non-failing `advisories` list, with reason, known
prefix, and helper lines when available. Existing `fail` and `ignore` choices
keep their behavior; query annotation's default name for `fail` is `report`.
Proven violations continue to fail even when the same call has residual
uncertainty. `no-mistakes` file and line suppression directives apply to
advisories as they do to findings.

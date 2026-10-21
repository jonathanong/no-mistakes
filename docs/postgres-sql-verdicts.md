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
  uncertainty prevents either conclusion. A parse failure outside every
  conflict statement does not block a clear result for
  `postgres-conflict-ordering`: that failure cannot hide an `ON CONFLICT`.
  For `postgres-conflict-ordering`,
  `unresolved-arbiter`, `ambiguous-arbiter`, and `unresolved-source-order` are
  still failing check findings; their ledger row stays `abstain`
  (`unresolved-conflict-catalog-or-source`) because the catalog or source shape
  proves neither a violation nor a clear result.
  A multi-row `ON CONFLICT DO NOTHING` with no conflict target is a `violation`
  on a verified prefix: a trailing interpolation cannot add that target. The
  row keeps `verified-prefix` evidence and `opaque-sql-tail` as residual
  uncertainty. An order finding still abstains, because the tail can supply
  the missing `ORDER BY`.

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
residual uncertainty in the non-failing `advisories` list. Each executor call,
rule, configured application, and executor profile produces at most one residual
advisory. Its `target` lists unresolved checks in sorted, comma-separated
order and may name more than one check. The message combines sorted,
deduplicated checks, residual reasons, known prefixes, and helper lines. A
single subcheck keeps its existing target. When any grouped obligation is a
violation, the message says the call proved a violation and that remaining SQL
is unresolved. An abstain-only group says the call has unresolved SQL.
The optional ledger remains one row per obligation.

Advice is attributed to the executor call's starting line, including multiline
finite alternatives and imported SQL. The original half-open call span remains
the aggregation identity, so separate calls on one line stay separate. Proven
findings retain their existing SQL branch/helper origin lines. Fail-mode parse
errors retain the invalid alternative's origin line as well. Suppress advice
at its executor call line; helper lines remain contextual evidence in its message.
This avoids moving existing failures when a call also has unresolved SQL.

Existing `fail` and `ignore` choices keep their behavior; query annotation's default name for `fail` is `report`.
Proven violations continue to fail even when the same call has residual
uncertainty. `no-mistakes` file and line suppression directives apply to
advisories as they do to findings.

Application identity borrows the request's prepared facts and report-local sink.
Creating another configured application does not copy the prepared file/TS
containers or recount unproven member queries. Concurrent applications retain
separate configured indices and names; separate check reports retain separate
sinks. Actual supplemental fact/catalog projections still compose their own
requested view. The disabled ledger returns before constructing an application
view or allocating its name.

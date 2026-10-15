# Default export execution frequency

Issue: <https://github.com/jonathanong/no-mistakes/issues/2017>.
Base: `ac03f595020283283c6a8fda82472aef05365389` (merged #2014).

The collector assigns a synthetic callable owner to a default export expression.
That ownership does not make the expression deferred. A collector-only optional
module owner now identifies this exact synthetic scope while collecting facts.
The existing `ValueCall.once` field accepts this owner as well as ordinary module
context, retaining its direct-arrow and repeated-execution guards. The field is
restored after traversal; nested callable contexts retain distinct parser IDs.
The synthetic ID starts at the export keyword, before every real callable in the
expression. A saved-fixture extraction test asserts the distinct ordered IDs,
their exact context chain, and the outer/inner frequency distinction.

There is no new AST pass, retained per-call fact field, resolver cache, source
read, or projection allocation. Prepared facts and the canonical graph remain
shared. File-qualified receiver ownership and acquisition guards are unchanged.

Saved physical fixtures cover direct default IIFEs and object-member IIFEs plus
write-first, deferred arrow/function/method, nested callable, loop, instance field,
auto-accessor, async, and recursive controls. Existing graph, CLI, and Rust N-API
tests consume these fixtures. The compiled CommonJS/ESM asynchronous
`analyzeProject` parity test also covers these cases and existing cross-file
timing controls. CLI, graph, and Node API documentation describe the behavior.

## Validation and publication

Before edits and immediately before publication, actual issue fetches confirmed
the issue open. `github_search_prs` found no duplicate implementation; the only
relevant all-state result was merged #2014. Root reviewed the production delta
and span proof before authorizing a draft PR for remote validation.

Passed locally: 17 configured AST rule tests using CI's
`--skip-snapshot-tests`, AST scan, JavaScript syntax/format/lint, documentation
spelling, and whitespace checks. The supplied historical CLI returned no
configured check findings, warnings, or advisories; it is not candidate semantic
validation. Unrelated pre-existing formatting in the default helper is retained
to avoid adding untested formatting-only lines to patch coverage.

Historical CLI scoping found no Rust include-file dependents or Cargo targets,
without fallback. Signature-impact inventory hit the existing invalid-UTF8
fixture and cannot establish complete signature impact. Impacted checks reported
the existing unresolved Vitest setup warning. Changed JavaScript and positive
fixture imports resolved successfully.

No local compiler, linker, native test, benchmark, or protected artifact cleanup
was run because the shared environment had about 1.7 GB free. Full remote semantic
tests, actual compiled-addon parity, quality and coverage gates, external Codecov
statuses, performance review, and current-head review remain pending. The PR
stays draft until those gates pass. No performance improvement or merge is claimed.

## First draft CI correction

The first draft at `b1c7226` failed Rust test compilation and lint checks because the
new extraction assertion used `ParserReturn.errors`; this parser exposes
`diagnostics`. The correction uses the neighboring test convention. No analyzer
source changes are included in this correction.

Inspection also found that the existing inline-owned-member parity script was
omitted from CI's explicit compiled-addon invocation. The ordinary JavaScript
run skips its native tests. This PR now requires the script from the real N-API
test entrypoint, executed with the actual addon environment variable set on
Linux, macOS, and Windows. Both tests assert a CJS
promise, await CJS and ESM results, and compare all expected call-site counts.
Prior #2014's manual compiled-addon reproduction does not establish that CI ran
this previously omitted script. Fresh executed native results remain pending.

## Computed member callable guard

Current-head Codex review found an introduced unsafe proof: specialized object
function/arrow and anonymous class method visitors do not push a display scope
for dynamic computed keys, leaving the synthetic module owner visible inside
their deferred bodies. The correction wraps the three actual callable traversals
in the existing repeated-execution guard, including default parameters, after
computed key and decorator evaluation. It changes no graph/export/resource
ownership and adds no collector field, fact field, allocation, pass, or cache.

Saved object arrow/function/method, class method, and default-parameter controls
remain unknown. Computed object/class keys and an ordinary module computed-arrow
control are covered across graph, CLI, Rust N-API, and compiled CJS/ESM tests.
A saved extraction fixture alternates deferred bodies with module-evaluated keys
and the final value, asserting guard restoration after all three visitor paths.
The review thread remains open pending actual current-head validation.

## Historical validated head and required main integration

At `e7d26f8`, all 30 CI jobs passed, including full Rust semantic tests,
documentation tests, CRAP/complexity and unchanged coverage floors, plus actual Linux,
macOS and Windows native parity. All four external Codecov checks passed
(Rust patch 100%, project 99.46%; JavaScript patch unaffected, project 99.76%).
Exact-head Codex found no major issues; the corrected computed-member safety
thread was resolved after actual tests. Later duplicate #2026 was closed unmerged
with its branch preserved; #2020 remains canonical for #2017.

The initial expected-base CodSpeed report flagged non-TS
`language_frontends/extract` tracked peak heap at 210 KB to 482.1 KB. One
same-head `ubuntu-24.04-arm` memory-shard rerun passed: 220 untouched and four
skipped benchmarks. Its exact heap was omitted, so no value is inferred. The
read-only profile response linked below observed an exact-head stored peak of
375,043 B, including 160,008 B retained in unchanged Kotlin regex lazy-DFA worker
caches; the remaining 215,035 B equals the base work peak. Worker ordering is
inferred. The profile does not match the original 482.1 KB report, so attribution
of that exact original value is unproven. This metric is tracked heap, not RSS;
system call warnings establish no cause. No runtime fix or improvement is claimed.

- Original run: `38659029493`, attempt 1, memory job `116178414390`.
- One rerun: same run/head, attempt 2, memory job `116180292392`.
- Profile: <https://github.com/jonathanong/no-mistakes/pull/2020#issuecomment-6181574412>.
- Existing harness follow-up: <https://github.com/jonathanong/no-mistakes/issues/2025>.

The final readiness check found genuine conflicts after main advanced to
`d36ea2af42d7a2a6d20074e4cd8ac1d0bc89568a`. This merge preserves main's loop-setup
precision, argument-evaluation facts and all existing default/computed safety
controls. Conflict resolutions retain both N-API includes, both native test
sections and their shared bridge registration, and combined public documentation.
No additional analyzer behavior is introduced beyond the two integrated parents.
The historical results above do not validate the integrated tree: fresh full CI,
actual external coverage/performance gates and exact-head review remain required.

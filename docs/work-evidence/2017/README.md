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

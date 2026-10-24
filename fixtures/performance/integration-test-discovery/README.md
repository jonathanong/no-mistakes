# Integration test discovery demand

The saved policies target one identical frozen Vouchington source corpus.
`consuming.yml` narrows recursive test discovery to explicit integration policy
includes/excludes. Helpers outside those globs still retain their imports,
exports, function summaries, and parse errors. `full-discovery-control.yml`
consumes every source file; `non-consuming-control.yml` requests an import graph
without integration annotation facts.

Run baseline and candidate default-feature release binaries against the same
corpus with `check --config <policy> --format json`, at jobs 1, 4, and default.
Measure observer off and `--timings --verbose-timings` separately, using fresh
processes and balanced ABBA order. Preserve report bytes and statuses as well as
wall/CPU/RSS samples. The regression fixture in
`test-cases/integration-tests/discovery-scope/fixture` locks mixed-framework
includes/excludes, imported helper resolution, nested tests, and synthetic
function-name collisions. Its sibling `discovery-scope-errors/fixture` retains
out-of-suite parse failures.

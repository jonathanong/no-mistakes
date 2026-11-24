# Mixed query projection workload

The materializer assembles the saved graph, query, test-scan, dependency,
Storybook, and workflow fixtures used to inspect the prepared rule phases.
Use the same source checkout and generated directory for both binaries.
The corrected config scopes the Storybook and Playwright checks to their
projects. The no-scan config isolates the cost of the test-scan rule.

With Python and PyYAML 6.0.1 installed, run from the repository root.

```sh
scratch=$(mktemp -d)
python3 fixtures/performance/query-projection-mixed/materialize.py . "$scratch"
no-mistakes --jobs 8 check --root "$scratch/mixed" --config "$scratch/mixed-corrected.yml" --timeout 0 --lock-timeout 0 --format json --verbose-timings
```

Use a separate fresh process per sample. Drop `--verbose-timings` for
wall-time measurements. The work counters describe visits by the query
projection and its callers. Phase timings can overlap under parallel
execution, so use them to locate work during a run.

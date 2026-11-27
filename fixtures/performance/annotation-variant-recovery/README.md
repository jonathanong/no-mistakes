# SQL variant recovery at annotation scale

The generator creates 2,300 modules that import a configured executor and a
shared three-function callback chain. Each consumer has eight exported
functions with conditional SQL text. Another 4,000 modules have no
executor imports. The 6,301 TypeScript modules contain 298,004 physical lines
and 12,416,941 bytes of source without JSX. The low and max configurations
share their executor settings, include globs, and unanalyzable policy.

The generated project has one deliberately unannotated direct call at
`src/root-0000.mts:5`. At both efforts the check must exit 1 with that finding
and empty advisory and warning arrays. Compare complete JSON and exit status
for the same effort across binaries.

```sh
fixture=$(mktemp -d)
python3 fixtures/performance/annotation-variant-recovery/materialize.py "$fixture"
for effort in low max; do
  /usr/bin/time -f 'wall=%e user=%U system=%S rss=%M' \
    no-mistakes --jobs 4 check --root "$fixture" --config "$fixture/$effort.yml" \
    --timeout 0 --lock-timeout 0 --format json > "$fixture/$effort.json"
done
```

Use the same checkout of the generated source, release flags, and worker
count. Alternate before and after runs with timing diagnostics off. Compare
the max-to-low wall and total CPU ratios separately. The end-to-end ratio is
a measurement on the current host.

The deterministic ceiling is exercised by the
`embedded::walk::variants::limit::tests` unit tests. Comparing equal variants
allocates zero times. Joining matching binary alternatives or retaining
sixteen distinct alternatives allocates at most four times for the result
vectors. The assertions count allocation requests on the test thread and
verify that choice identity, source provenance, and ordering survive.

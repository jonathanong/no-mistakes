# Suppression index benchmark

Each subroot runs one `forbidden-calls` application scoped to `src/worker.ts`,
with file traversal and the exact target `scheduler.register`. Each saved
source contains 3,000 static member calls and uses only `example.invalid`.
The rule hands its findings to central suppression. The dense case distributes
line directives through the source, including late in the file. The controls
keep the same call density.

| Subroot | Source lines | Source bytes | Findings entering suppression | Reported | Suppressed | Exit |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `dense` | 3,105 | 217,605 | 3,000 | 2,800 | 200 | 1 |
| `no-directives` | 3,005 | 204,105 | 3,000 | 3,000 | 0 | 1 |
| `single` | 3,005 | 201,171 | 1 | 0 | 1 | 0 |
| `file-disable` | 3,006 | 204,180 | 3,000 | 0 | 3,000 | 0 |
| `long-header` | 5,006 | 339,678 | 1 | 0 | 1 | 0 |
| `early-quoted-marker` | 3,006 | 201,199 | 1 | 1 | 0 | 1 |

The dense case consumes 100 same-line and 100 next-line directives. The
single case calls the allowed `scheduler.observe` method 2,999 times, then
suppresses the final registration on line 3,005. The file-wide suppressions
all point to line 1. The long-header case puts its file directive before
2,000 schedule manifest comments and its only registration on line 2,007.
The early-quoted-marker case places its only registration on line 6 and the
directive text in a string on line 3,005. Unused-directive auditing is disabled
throughout to isolate suppression matching.

## Run

From the repository root, set `bin` to the absolute path of the release
binary being measured. Use the same absolute subroots for every binary.

```bash
fixture="$PWD/fixtures/performance/suppression-index"
out="$(mktemp -d)"
for mode in dense no-directives single file-disable long-header early-quoted-marker; do
  status=0
  /usr/bin/time -f '%e %U %S %M' -o "$out/$mode.time" \
    "$bin" check --root "$fixture/$mode" --jobs 1 --format json \
    --include-suppressed >"$out/$mode.json" 2>"$out/$mode.stderr" || status=$?
  printf '%s\n' "$status" >"$out/$mode.status"
  jq '{reported: (.rules | length), suppressed: (.suppressed | length),
       directives: ([.suppressed[]? | .directive.kind] |
                    group_by(.) | map({kind: .[0], count: length}))}' \
    "$out/$mode.json"
done
```

The timing columns are elapsed seconds, user CPU seconds, system CPU seconds,
and peak RSS in KiB. Compare stdout, stderr, and exit status between binaries.
Repeat with `--include-suppressed` omitted to match the default report path
used by `benchmark_support::check_json`.

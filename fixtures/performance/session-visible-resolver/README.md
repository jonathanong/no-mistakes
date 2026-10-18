# Session resolver controls

Run both configurations with the repository root, not this directory:

```sh
no-mistakes check --root "$repo" --config fixtures/performance/session-visible-resolver/consuming.yml --format json --jobs 4
no-mistakes check --root "$repo" --config fixtures/performance/session-visible-resolver/control.yml --format json --jobs 4
```

The consuming configuration enables unique exports, queue extraction, and an
explicit integration suite over real Node package files. The control requests
an import-graph check without those domains. Both carry project include and
skip-directory payloads and use one unchanged physical repository inventory.
Source size, findings hashes, exit status, and exact revisions belong in the
measurement record; fixture file count alone does not describe the workload.

Also measure the existing `core-analysis` and `graph-gates` roots, which contain
TSX, queue factories/workers, runner configurations, and configured includes.
Keep diagnostic off/on samples separate, and match jobs 1, 4, and default.
Use balanced baseline/candidate/candidate/baseline order and record raw wall
and CPU time plus peak RSS. Existing check findings may produce exit status 1.

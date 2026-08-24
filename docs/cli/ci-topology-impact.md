# CI topology impact

`no-mistakes ci topology-impact` compares two exact revisions without
shelling out, and produces a deterministic fail-open routing report for one
entry workflow.

```sh
no-mistakes ci topology-impact --root . --base "$BASE_SHA" --head "$HEAD_SHA" --entry-workflow ci.yml
```

The output is schema version 1 and includes normalized revision IDs, changed
paths, affected workflows and root job IDs, diagnostics, and `globalFallback`.
Consumers must treat `globalFallback: true` or a missing report as permission
to run every otherwise eligible CI producer.

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

`affectedRootJobIds` contains directly affected entry jobs plus only their
transitive `needs` prerequisites. It intentionally excludes downstream
aggregate jobs and their unrelated sibling prerequisites. `affectedWorkflows`
includes every changed recognized workflow descriptor from either revision and
its reusable-workflow caller closure, even when that descriptor is standalone.

Each diagnostic carries a `scope`. A `localized` diagnostic includes sorted
`rootJobIds` only when every implicated endpoint maps to entry-workflow jobs;
all parser, Git, malformed, ambiguous, and unbound cases are `global`.

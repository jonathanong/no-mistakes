# `no-mistakes importers`

List the files that directly import a single file, with a dependents count.

```sh
no-mistakes importers src/utils.mts --format json
no-mistakes importers src/utils.mts --tests --format json
```

Use this for the quick "who imports this one file?" question without formulating
a structural traversal. It runs a single reverse import scan (no full graph
build), so it returns fast on a mid-size monorepo. `directImporters` lists the
direct importer files and `dependentsCount` summarizes them.

`--tests` additionally computes the transitive impacted-test set (this builds the
dependency graph, so it is the slower path) and reports it as `testImpact`.

For transitive or symbol-level impact, or to follow non-import edges, use
[`dependents`](dependents.md) instead.

Key options: `--root`, `--tsconfig`, `--tests`, `--format`, and `--json`.

Node API: `importers(options)`.

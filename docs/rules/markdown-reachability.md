# `markdown-reachability`

Enforces a small, explicit documentation-discovery graph: configured Markdown
targets must be reachable from an instruction root. The default allowed paths
are a direct link from `CLAUDE.md`, or one `README.md` intermediary:
`CLAUDE.md -> README.md -> doc`.

```yaml
rules:
  - rule: markdown-reachability
    scope: repository
    include: ["**/*.md"]
    options:
      rootFilenames: [CLAUDE.md]
      indexFilenames: [README.md]
      maxDepth: 2
```

Counterexample: `CLAUDE.md -> overview.md -> detail.md`. The intermediary is
not a configured `README.md`, so `detail.md` is not discoverable under this
rule.

Only tracked local `.md` link destinations count. Query strings and fragments
are ignored while resolving a link; external, fragment-only, and escaping links
do not count. Inline and reference links are supported. Directories are never
implicitly resolved to `README.md`.

Finding paths are lexical from the request root. External configured projects
use a stable `../project/file.md` path, so standard file suppressions resolve
back to the source. Baseline entries remain relative to each effective project
root, keeping a project's rollout state portable across request locations.

For a staged rollout, `baselineFile` may name a tracked JSON object mapping a
path to `{ "state": "depth", "depth": N }` or
`{ "state": "unreachable" }`. Entries must exactly match current violations;
resolved, changed, malformed, and deleted entries fail as stale.

Fix: add a direct link from a root or a direct root-to-README and README-to-doc
path. Prefer removing baseline entries as documents are repaired.

Suppressions use standard `no-mistakes` directives. Prefer fixing the map;
suppression hides future discoverability regressions.

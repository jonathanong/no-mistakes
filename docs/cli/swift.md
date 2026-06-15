# `no-mistakes swift`

Query the Swift package graph: which files import a Swift file, and which test
targets cover it. Built on the Swift facts already integrated into the dependency
graph (`swift-import`, `swift-ref`, `swift-package` edges).

Configure the analyzed packages via `tests.swift.packages`; no Swift analysis
happens without it.

```yaml
tests:
  swift:
    packages:
      - swift-clients/core
      - swift-clients/ui
```

## Subcommands

| Command | Purpose |
| --- | --- |
| [`swift importers`](swift-importers.md) | Swift files that import or reference a Swift file. |
| [`swift test-targets`](swift-test-targets.md) | SwiftPM test targets that transitively cover a Swift file. |

See [`Graph edges`](../graph-edges.md) for the Swift edge kinds these queries use.

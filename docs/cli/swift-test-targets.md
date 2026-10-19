# `no-mistakes swift test-targets`

List the SwiftPM test targets that transitively cover a given Swift source file,
following `swift-import`, `swift-ref`, and `swift-package` edges to files that
belong to a test target.

```sh
no-mistakes swift test-targets swift-clients/core/Sources/CoreAPI/Endpoint.swift --json
```

Each result is a test target, its package directory, and the command that runs
just that target. That command is `swift test --package-path <package> --filter
<target>`, or the package's `tests.swift.baseCommands` argv in place of
`swift test`. The command uses `tests.swift.baseCommands` for a declared package
and `swift test` otherwise. Requires `tests.swift.packages` in config.

Node API: `swiftTestTargets(options)`.

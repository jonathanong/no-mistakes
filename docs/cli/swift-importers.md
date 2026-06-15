# `no-mistakes swift importers`

List the Swift source files that import or reference a given Swift file, following
`swift-import` and `swift-ref` edges (direct and transitive).

```sh
no-mistakes swift importers swift-clients/core/Sources/CoreAPI/Endpoint.swift --json
```

Each result is an importing file plus its traversal depth (1 = direct). Requires
`tests.swift.packages` in config.

Node API: `swiftImporters(options)`.

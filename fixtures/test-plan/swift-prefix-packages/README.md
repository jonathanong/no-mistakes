# Swift package prefix

`clients/core` is listed before `clients/core-extra`, and only `clients/core`
has `baseCommands`. The `extra` policy include is under `clients/core-extra`.
A raw string prefix would claim that policy for `clients/core` and render
`/opt/core-only-swift`. The plan must render `swift test --package-path
clients/core-extra`.

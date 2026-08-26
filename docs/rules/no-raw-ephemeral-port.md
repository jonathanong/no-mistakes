# `no-raw-ephemeral-port`

## Suppression

Use a next-line directive for a one-off bind or a file directive for the
allocator implementation. Prefer the exact `allow` path list for known binders.

## Why and when

Use this rule when test or service runners allocate ports from a controlled
range and raw OS-selected ports would make parallel execution nondeterministic.

## What it catches

It catches literal port `0` binds in supported Python, shell, YAML, JavaScript,
and TypeScript forms, including Node `listen(0)` and `{ port: 0 }`.

## Options

`include` defaults to the supported source/config extensions listed below.
`allow` is a relative-path glob list for allocator implementations, and
`message` is an optional appended hint; no other rule-specific options exist.

## Valid example

`server.listen(listenPort)` passes because an allocator supplies the port
instead of a raw literal zero.

## Related rules

[`test-no-dependency-pins`](test-no-dependency-pins.md) prevents brittle test
assertions; this rule specifically protects runner-port allocation.

Bans raw ephemeral port 0 socket binds and Node `listen(0)` calls so tests
cannot occupy a deterministic runner port slice. Bind the host/port tuple
`.bind(("host", 0))` in Python, shell, and YAML, or call `listen` with a
numeric `0` / `{ port: 0 }` in JavaScript and TypeScript.

```yaml
rules:
  - rule: no-raw-ephemeral-port
    scope: repository
    options:
      include:
        - "**/*.py"
        - "**/*.ts"
      allow:
        - src/binder.ts
      message: use a configured allocator
```

`include` defaults to `*.bash`, `*.cjs`, `*.cts`, `*.js`, `*.jsx`, `*.mjs`, `*.mts`,
`*.py`, `*.sh`, `*.ts`, `*.tsx`, `*.yaml`, `*.yml`, and `*.zsh`.
`allow` skips binder implementations by relative path glob.
`message` appends an optional hint after the default finding text.

Counterexample: a test binds or listens on literal port 0.

```py
sock.bind(("127.0.0.1", 0))
```

```ts
server.listen(0);
server.listen({ port: 0 });
```

Fix: bind through an allowlisted allocator, or pass a non-zero / non-literal
port. Put the binder implementation itself in `allow`.

```ts
server.listen(listenPort);
```

Use `no-mistakes-disable-next-line no-raw-ephemeral-port` or
`no-mistakes-disable-file` for a one-off exception.

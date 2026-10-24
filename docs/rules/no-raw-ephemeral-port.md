# `no-raw-ephemeral-port`

Bans raw ephemeral port 0 socket binds and Node `listen(0)` calls so tests
cannot occupy a deterministic runner port slice.

## Why and when

Use this rule when test or service runners allocate ports from a controlled
range and raw OS-selected ports would make parallel execution nondeterministic.

## What it catches

It catches literal port `0` binds in supported Python, shell, YAML, JavaScript,
and TypeScript forms, including Node `listen(0)` and `{ port: 0 }`. A
parenthesized or TypeScript-asserted callee still counts, as do static string
and no-substitution template members such as `server["listen"](0)` and
``server[`listen`](0)``. Python tuple binds accept literal integer zero with
unary `+` or `-`, including base-prefixed zero spellings. Dynamic member names
and variable ports are not inferred. An assignment such as
`export const port = 0` is not a bind.

## Options

`include` defaults to the supported source/config extensions listed below.
`allow` is a relative-path glob list for allocator implementations, and
`message` is an optional appended hint; no other rule-specific options exist.

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

## Valid example

`server.listen(listenPort)` passes because an allocator supplies the port
instead of a raw literal zero.

```ts
server.listen(listenPort);
```

## Counterexample

A test binds or listens on literal port 0.

```ts
server.listen(0);
server.listen({ port: 0 });
```

```py
# bind.py
sock.bind(("127.0.0.1", 0))
```

## Fix

Bind through an allowlisted allocator, or pass a non-zero / non-literal port.
Put the binder implementation itself in `allow`.

## Analysis

### Effort

- Runs at `max` only. Configuring `effort: low` (on the rule, or as a repository or CLI default) is not an error: `check` records the warning `requested effort low; effective effort max because this rule has no low implementation` and runs the rule at `max`.

### Unanalyzable inputs

Upgrade note: in-scope files that cannot be read as UTF-8 now fail `check`
under the default `unanalyzable: report` instead of silently passing. Fix or exclude those inputs, or set
`options.unanalyzable: advise` or `ignore` to restore the previous successful
exit status for uncertainty alone. Independent definite findings still fail
under every policy.

Selected files that cannot be read as UTF-8 produce an `unanalyzable`
diagnostic instead of silently passing. Restore read access or repair the
encoding so the rule can verify the file. The shared `options.unanalyzable`
setting inherits the repository policy: `report` (the default) fails,
`advise` emits non-failing advice, and `ignore` omits uncertainty. Definite
violations remain failures under every policy, including when `effort: low`
falls back to `max`.

Configured scope, includes, excludes, and rule-specific exemptions still
select the inputs. Suppressions apply to both channels, including directives
retained in an invalid UTF-8 read error; source bytes are never reinterpreted
as analyzable code. Rule-level custom messages apply to these diagnostics.

## Suppression

Use `no-mistakes-disable-next-line no-raw-ephemeral-port` or
`no-mistakes-disable-file` for a one-off exception. Prefer the exact `allow`
path list for known binders.

## Related rules

[`test-no-dependency-pins`](test-no-dependency-pins.md) prevents brittle test
assertions; this rule specifically protects runner-port allocation.

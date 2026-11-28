# `no-mistakes/test-no-fixed-temp-paths`

## Why

A test that writes to a fixed temporary path can collide with another worker
or reuse files from an earlier run. A unique directory gives each test
execution its own files.

## What it catches

The rule reports fixed paths under `/tmp/` or `/var/tmp/` only when they reach
Node filesystem or process calls. It checks path arguments to `fs` and
`fs/promises` functions, and commands, argument lists and `cwd` options passed
to `child_process` spawn/exec functions. Data, expected values, environment
strings and mocked object-store keys are not filesystem paths merely because
they contain `/tmp/`.

Paths can be string literals, untagged expression-free templates, or static
Node `path.join` / `path.resolve` results. The rule follows chains of stable
local `const` aliases for sink arguments and for the root of a join or resolve call.
The segments after the root must be literal strings, with at least one
non-empty segment.

A file that mocks `fs`, `node:fs`, `fs/promises`, or `node:fs/promises` through
Vitest's `vi.mock` or Jest's `jest.mock` is exempt, even if the mock call follows
the filesystem use. That exemption also drops an `unanalyzable` report for a
call whose filesystem binding is not static.

## Disallowed

```ts
// example.test.ts
import { tmpdir } from "node:os";
import { join } from "node:path";
import { writeFile } from "node:fs/promises";

await writeFile("/tmp/example.json", "{}");
await writeFile(join(tmpdir(), "example.json"), "{}");

const root = "/var/tmp/example";
await writeFile(join(root, "result.json"), "{}");
```

The rule resolves named, default, and namespace imports from Node's built-in
modules, along with static CommonJS requires, TypeScript import-equals
declarations, and simple local `const` aliases. It uses lexical scope to
resolve the bindings.

## Allowed

Use `mkdtemp`, `mkdtempSync`, `mkdtempDisposable`, or `mkdtempDisposableSync`
for a unique directory. Prefixes passed directly or through local `const`
aliases are allowed. Disposable variants also support `using` /
`await using` declarations when the runtime supports them.

```ts
// example.test.ts
import { tmpdir } from "node:os";
import { join } from "node:path";
import { mkdtemp, rm, writeFile } from "node:fs/promises";

const directory = await mkdtemp(join(tmpdir(), "example-"));
try {
  await writeFile(join(directory, "result.json"), "{}");
} finally {
  await rm(directory, { recursive: true, force: true });
}
```

Disposable cleanup also works with a constant prefix.

```ts
// example.test.ts
import { mkdtempDisposableSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";

const prefix = join(tmpdir(), "example-");
using directory = mkdtempDisposableSync(prefix);
```

## Options

Enable this opt-in rule with `"no-mistakes/test-no-fixed-temp-paths": "error"`.
It checks `.test` and `.spec` files with JS, TS, JSX, TSX, and their
CommonJS and ESM extensions. It is disabled by default in both presets.

`unanalyzable` is `"finding"` or `"ignore"` and defaults to `"finding"`. A
filesystem call whose `fs` binding comes from a non-static `require` reports
`unanalyzable`.

`fs.writeFileSync`, `const { ["writeFileSync" as const]: write } = fs`, and
`(require as NodeRequire)("node:fs")` are the same filesystem binding.
`loaded.promises.writeFile` and `const { promises } = require(spec)` keep an
unresolved `fs` binding through `promises`. `loaded.lstatSync` from that require reports `unanalyzable`, as `statSync` does. A destructured name that is not a
filesystem method, such as `const { test } = require(spec)`, stays quiet. A computed member of a static `fs`, `fs/promises`, `path`, or `os` binding, such as `fs[method]("/tmp/result", "")`, stays silent. A computed destructure of that module, such as `const { [method]: write } = fs`, stays silent too.

## Fix

Create a fresh directory in the test or its per-test setup with `mkdtemp`
or a disposable variant, and build file paths under the returned directory.
Remove the directory during test cleanup.

## Suppression

Use `no-mistakes-disable-next-line` for an intentional fixed path.

```ts
// example.test.ts
import { readFile } from "node:fs/promises";

// no-mistakes-disable-next-line test-no-fixed-temp-paths -- verifies a legacy filesystem contract
await readFile("/tmp/example.json");
```

Use `no-mistakes-disable-line` on the finding line or `no-mistakes-disable-file`
before the first code token to apply a broader suppression.

## Limits

- Concatenation such as `tmpdir() + "/result.json"`, templates with
  interpolations, and `new URL("file:///tmp/result.json")` fall outside the check.
- The rule leaves shell syntax, runtime path segments, and paths
  from helper functions, mutable bindings, or imports unanalyzed.
- In a checked argument position, any literal beginning with `/tmp/` or
  `/var/tmp/` can produce a finding, including non-path text passed in a
  process argument.
- Only `vi.mock` and `jest.mock` trigger the file-wide mock exemption.
  `vi.doMock` and `jest.doMock` leave filesystem calls subject to the check.
  The exemption covers tests that later restore real filesystem methods.

## Related rules

- [`test-no-shared-state`](test-no-shared-state.md) identifies shared mutable
  state that can make tests order-dependent.

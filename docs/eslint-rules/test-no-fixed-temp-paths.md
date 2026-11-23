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
Node `path.join` / `path.resolve` results. A single local `const` alias can
connect a path to its sink. Joins with non-literal segments and the bare
temporary root, including joins with only empty segments, are left alone.

A file that mocks `fs`, `node:fs`, `fs/promises`, or `node:fs/promises` through
Vitest's `vi.mock` or Jest's `jest.mock` is exempt, even if the mock call follows
the filesystem use.

## Disallowed

```ts
// example.test.ts
import { tmpdir } from "node:os";
import { join } from "node:path";
import { writeFile } from "node:fs/promises";

await writeFile("/tmp/example.json", "{}");
await writeFile(join(tmpdir(), "example.json"), "{}");
```

The rule resolves named, default, and namespace imports from Node's built-in
modules, along with static CommonJS requires, TypeScript import-equals
declarations, and simple local `const` aliases. It uses lexical scope to
resolve the bindings.

## Allowed

Use `mkdtemp`, `mkdtempSync`, `mkdtempDisposable`, or `mkdtempDisposableSync`
for a unique directory. Prefixes passed directly or through a single local
`const` alias are allowed. Disposable variants also support `using` /
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

Disposable cleanup also works with a constant prefix:

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
CommonJS and ESM extensions. This rule has no options. It is disabled by default in both presets.

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

This is a file-local static check. It follows one `const` path alias and known
Node APIs, rather than arbitrary wrappers, returned values, reassigned bindings,
or imported path values. It does not interpret shell commands or expand runtime
path segments. A dynamic path may still collide; create a unique directory
instead of relying on the rule to prove it safe. Mock exemptions apply to the
whole file, including tests that restore real filesystem methods later.

## Related rules

- [`test-no-shared-state`](test-no-shared-state.md) identifies shared mutable
  state that can make tests order-dependent.

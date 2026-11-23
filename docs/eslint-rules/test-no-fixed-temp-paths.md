# `no-mistakes/test-no-fixed-temp-paths`

## Why

A test that writes to a fixed temporary path can collide with another worker
or reuse files from an earlier run. A unique directory gives each test
execution its own files.

## What it catches

The rule reports string literals and untagged, expression-free template
literals whose value starts with `/tmp/` or `/var/tmp/`. It also reports Node's
`path.join(os.tmpdir(), ...)` when every segment after `os.tmpdir()` is a
string literal or expression-free template. Joining the temporary root by
itself also produces a shared path.

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

Pass the temporary prefix directly as the first argument of Node's `mkdtemp`
or `mkdtempSync`. Both create a unique directory from the prefix.
The rule also leaves joins with dynamic segments alone.

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

## Options

Enable this opt-in rule with `"no-mistakes/test-no-fixed-temp-paths": "error"`.
It checks `.test` and `.spec` files with JS, TS, JSX, TSX, and their
CommonJS and ESM extensions. This rule has no options. It is disabled by default in both presets.

## Fix

Create a fresh directory in the test or its per-test setup with `mkdtemp`
or `mkdtempSync`, and build file paths under the returned directory.
Remove the directory during test cleanup.

## Suppression

Use `no-mistakes-disable-next-line` for an intentional fixed path.

```ts
// example.test.ts
// no-mistakes-disable-next-line test-no-fixed-temp-paths -- verifies a legacy path format
const legacyPath = "/tmp/example.json";
```

Use `no-mistakes-disable-line` on the finding line or `no-mistakes-disable-file`
before the first code token to apply a broader suppression.

## Related rules

- [`test-no-shared-state`](test-no-shared-state.md) identifies shared mutable
  state that can make tests order-dependent.

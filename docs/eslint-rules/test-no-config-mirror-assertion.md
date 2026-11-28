# `no-mistakes/test-no-config-mirror-assertion`

## Why

A test that reads a tracked repository file by a static path and asserts literal
strings or values in its contents mirrors the file instead of verifying
behavior. It fails only when someone edits that file on purpose, and then the
test is edited to match. This rule is opt-in and is disabled by default in both
presets; enable it at `"warn"` for advisory adoption or `"error"` to enforce.

## What it catches

In JavaScript and TypeScript files named `*.test.*` or `*.spec.*`, the rule
tracks reads of static, non-fixture repository paths through:

- `readFileSync` and `readFile` from `fs`, `node:fs`, `fs/promises`,
  `node:fs/promises`, and `fs-extra` (including `fs.promises.readFile`,
  callback `readFile`, `.then(...)` handlers, `util.promisify(readFile)`,
  CommonJS `require`, renamed imports, and single-return same-file read
  helpers).
- Static paths built from string literals, expression-free templates,
  `path.join`, `path.resolve`, `path.normalize`, `path.dirname`,
  `process.cwd()`, `__dirname`, `import.meta.dirname`,
  `new URL(..., import.meta.url)`, `fileURLToPath(...)`, and immutable local
  `const` aliases.
- Values derived from the read text through `JSON.parse`, `yaml` (`parse`,
  `parseDocument`, `parseAllDocuments`), `js-yaml` (`load`, `loadAll`),
  configured `parsers`, `.toString()`, `.trim()`, `.toJSON()`, static property
  access, destructuring, `Object.keys` / `Object.values` / `Object.entries`,
  and local aliases.

When a read value flows **only** into literal `expect(...)` assertions inside a
test (`toContain`, `toMatch`, `toEqual`, `toBe`, `toStrictEqual`, and their
`.not` and `.resolves` forms), each mirror assertion is reported.

The rule does **not** flag:

- **Cross-file invariants**, including assertions comparing a read file against
  a value read from another file and per-item assertions over a set enumerated
  from a second file.
- **Executed scripts**, when the test (or an enclosing `beforeAll` /
  `beforeEach` hook) runs the same path through `child_process` (`spawn`,
  `spawnSync`, `execFile`, `execFileSync`, `fork`, `exec`, `execSync`).
- **Fixture files**, whose normalized path contains any configured
  `fixtureDirs` segment (default: `fixtures`, `__fixtures__`, `test-cases`).
  Parent traversal that leaves a fixture directory (such as
  `fixtures/../.github/workflows/ci.yml`) is still reported.
- **Files written by the test itself** (via `writeFile*`, `appendFile*`,
  `copyFile*`, `createWriteStream`) or created in temporary directories
  (`mkdtemp*`, `tmpdir()`, `/tmp/`, `/var/tmp/`).
- Files that mock `fs`, `fs/promises`, or `fs-extra` with `vi.mock` or
  `jest.mock`, or tests that pass the read text into production code under test.

## Disallowed

```ts
// ci.test.ts
import { readFileSync } from "node:fs";
import { expect, it } from "vitest";

it("disables persist-credentials in CI", () => {
  const text = readFileSync(".github/workflows/ci.yml", "utf8");
  expect(text).toContain("persist-credentials: false");
});
```

Parsed config reads that only assert literal fields are also reported:

```ts
// config.test.ts
import { readFileSync } from "node:fs";
import yaml from "yaml";
import { expect, it } from "vitest";

it("sets timeout", () => {
  const ci = yaml.parse(readFileSync(".github/workflows/ci.yml", "utf8"));
  expect(ci.jobs.test["timeout-minutes"]).toBe(15);
});
```

## Allowed

Compare two tracked files to enforce a cross-file invariant, or pass bounded
fixture files into the code under test:

```ts
// ci-consistency.test.ts
import { readFileSync } from "node:fs";
import yaml from "yaml";
import { expect, it } from "vitest";

it("runs CI for every workspace package", () => {
  const ci = readFileSync(".github/workflows/ci.yml", "utf8");
  const workspace = yaml.parse(readFileSync("pnpm-workspace.yaml", "utf8"));
  for (const pkg of workspace.packages) {
    expect(ci).toContain(pkg);
  }
});
```

Tests that execute a script or read from a fixture directory are also allowed:

```ts
// script.test.ts
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { expect, it } from "vitest";

it("runs the check script", () => {
  const scriptPath = "scripts/check.sh";
  const text = readFileSync(scriptPath, "utf8");
  const result = spawnSync("bash", [scriptPath], { encoding: "utf8" });
  expect(result.status).toBe(0);
  expect(text).toContain("set -euo pipefail");
});
```

## Fix

Choose one of:

1. **Delete the mirror test** when the configuration file is already its own
   source of truth.
2. **Replace it with a static rule** (for example, a dedicated `no-mistakes`
   workflow or structured-config policy rule) if the invariant must hold across
   files.
3. **Assert a cross-file invariant** against a second source of truth instead
   of hardcoding literals from one config file.

## Options

The rule is disabled by default in both presets. Configure it with `"warn"` for
advisory mode or `"error"` to fail lint:

```js
const noMistakes = require("eslint-plugin-no-mistakes");

module.exports = [
  {
    files: ["**/*.test.*", "**/*.spec.*"],
    plugins: { "no-mistakes": noMistakes },
    rules: {
      "no-mistakes/test-no-config-mirror-assertion": [
        "warn",
        {
          fixtureDirs: ["fixtures", "__fixtures__", "test-cases"],
          parsers: [{ module: "toml", name: "parse" }],
        },
      ],
    },
  },
];
```

- `fixtureDirs` (`string[]`, optional) replaces the default
  `["fixtures", "__fixtures__", "test-cases"]` directory segment list exempted
  as test fixtures.
- `parsers` (`{ module: string; name: string }[]`, optional) adds imported
  parser functions treated like `JSON.parse` and `yaml.parse`. Each entry sets
  `module` (the exact import specifier) and `name` (the imported export name,
  or `"default"` for a default-exported parser function).

## Suppression

Because some literal guards (such as security settings in GitHub Actions
workflows) are kept intentionally, this rule supports `no-mistakes-disable-*`
directives **with a required reason** after `--`. Directives without a nonempty
reason do not suppress findings.

```ts
// ci-security.test.ts
import { readFileSync } from "node:fs";
import { expect, it } from "vitest";

it("keeps persist-credentials disabled", () => {
  const text = readFileSync(".github/workflows/ci.yml", "utf8");
  // no-mistakes-disable-next-line test-no-config-mirror-assertion -- required workflow security guard
  expect(text).toContain("persist-credentials: false");
});
```

`no-mistakes-disable-line` on the assertion line, `no-mistakes-disable-next-line`
on the read declaration or assertion, and top-of-file `no-mistakes-disable-file`
all require a nonempty `-- <reason>`.

## Limits

- Dynamic paths from parameters, loops, or imported helpers outside the file
  are not flagged.
- Cross-file helper functions that wrap `readFileSync` in another module are
  outside single-file analysis.
- Only `vi.mock` and `jest.mock` trigger the file-wide filesystem mock
  exemption.

## Related rules

- [`test-no-live-repository-scan`](../rules/test-no-live-repository-scan.md)
  catches directory enumeration of the live repository from tests.
- [`test-no-fixed-temp-paths`](test-no-fixed-temp-paths.md) requires unique
  temporary directories in tests.

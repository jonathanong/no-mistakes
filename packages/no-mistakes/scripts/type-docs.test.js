const assert = require("node:assert/strict");
const test = globalThis.test || require("node:test").test;
const { readdirSync, readFileSync } = require("node:fs");
const { join } = require("node:path");

const packageRoot = join(__dirname, "..");

// Canonical JSDoc for the invocation-scope options that recur across
// `*Options` interfaces. Kept in one place so every occurrence in the
// shipped `.d.ts` files is required to match exactly, not just be "present".
// A stale/undocumented field here silently reproduces the incident this
// test guards against: a caller passed `tsconfig` in a monorepo without any
// type-level hint that it needed to be workspace-scoped.
const CANONICAL_JSDOC = {
  root: "/** Project root. Defaults to the current working directory. */",
  tsconfig: "/** Path to tsconfig.json for alias resolution. Searched upward if omitted. */",
  config:
    "/** Path to the no-mistakes config file (e.g. .no-mistakes.yml). Auto-discovered in root if omitted. */",
};

// Matches only bare `root?`/`tsconfig?`/`config?: string;` option fields.
// Report fields with a different type (e.g. `config?: string | null;` on
// `TestExecutionTarget`) don't match and are correctly left alone.
const FIELD_PATTERN = /^(\s*)(root|tsconfig|config)\?: string;\s*$/;

function declarationFiles() {
  return readdirSync(packageRoot).filter((name) => name.endsWith(".d.ts"));
}

test("every root/tsconfig/config option field carries its canonical JSDoc", () => {
  let checked = 0;

  for (const file of declarationFiles()) {
    const lines = readFileSync(join(packageRoot, file), "utf8").split(/\r?\n/);

    lines.forEach((line, index) => {
      const match = line.match(FIELD_PATTERN);
      if (!match) {
        return;
      }

      const [, indent, field] = match;
      const expected = `${indent}${CANONICAL_JSDOC[field]}`;
      const actual = lines[index - 1];

      assert.equal(
        actual,
        expected,
        `${file}:${index + 1}: \`${field}?: string;\` must be immediately preceded by:\n  ${expected}\ngot:\n  ${actual}`,
      );
      checked += 1;
    });
  }

  // Guards against the pattern silently matching nothing (e.g. a reformat
  // that changes field layout), which would make every assertion above
  // vacuous without failing the test.
  assert.ok(checked > 0, "expected to find at least one root/tsconfig/config option field");
});

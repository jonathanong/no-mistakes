import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { Linter } from "eslint";
import tsParser from "@typescript-eslint/parser";
import { it } from "vitest";
import { __dirname, plugin } from "./helpers.mjs";

const root = resolve(
  __dirname,
  "../../../test-cases/eslint-plugin/async-call-disposition/fixture/promise-handoff",
);
const targets = [{ sourceSpecifierPatterns: ["@app/jobs"], calleeNamePatterns: ["enqueueEmail"] }];
function reports(name, typed) {
  const filename = resolve(root, name);
  return new Linter({ cwd: dirname(filename) })
    .verify(
      readFileSync(filename, "utf8"),
      {
        files: ["**/*.ts"],
        languageOptions: {
          parser: tsParser,
          parserOptions: typed
            ? { project: resolve(root, "tsconfig.json"), disallowAutomaticSingleRunInference: true }
            : {},
        },
        plugins: { "no-mistakes": plugin },
        rules: { "no-mistakes/async-call-disposition": ["error", { targets }] },
      },
      { filename },
    )
    .map(({ line, messageId }) => [line, messageId]);
}

function lines(numbers) {
  return numbers.map((line) => [line, "disposition"]);
}

it.each([false, true])(
  "accepts a typed target handoff and keeps untyped slots (typed=%s)",
  (typed) => {
    assert.deepEqual(
      reports("caller.ts", typed),
      lines(typed ? [18, 23, 28, 32, 36] : [6, 11, 14, 18, 23, 28, 32, 36]),
    );
    assert.deepEqual(reports("final.ts", typed), []);
    assert.deepEqual(reports("dropped.ts", typed), typed ? lines([4]) : []);
  },
);

function oxlint(typeAware, files) {
  return spawnSync(
    process.execPath,
    [
      resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
      ...(typeAware ? ["--type-aware"] : []),
      "--deny-warnings",
      "--config",
      resolve(root, ".oxlintrc.json"),
      "--format",
      "json",
      ...files.map((name) => resolve(root, name)),
    ],
    { encoding: "utf8" },
  );
}

function diagnostics(result) {
  assert.equal(result.status, 1, result.stderr);
  return JSON.parse(result.stdout)
    .diagnostics.map((item) => [
      item.filename.replaceAll("\\", "/").split("/").at(-1),
      item.labels[0].span.line,
      item.code,
    ])
    .sort();
}

it("accepts the typed handoff under real type-aware Oxlint and reports a dropped call", () => {
  assert.deepEqual(
    diagnostics(oxlint(true, ["caller.ts", "dropped.ts", "final.ts"])),
    [
      ["caller.ts", 18],
      ["caller.ts", 23],
      ["caller.ts", 28],
      ["caller.ts", 32],
      ["caller.ts", 36],
      ["dropped.ts", 4],
    ]
      .map(([name, line]) => [name, line, "no-mistakes(async-call-disposition)"])
      .sort(),
  );
});

function withTypeAware(run) {
  const argv = process.argv;
  process.argv = argv.includes("--type-aware") ? argv : [...argv, "--type-aware"];
  try {
    return run();
  } finally {
    process.argv = argv;
  }
}

it("builds the type-aware program in-process when Oxlint passes --type-aware", () => {
  const proven = resolve(
    __dirname,
    "../../../test-cases/eslint-plugin/async-call-disposition/fixture/proven-consumers",
  );
  withTypeAware(() => {
    assert.deepEqual(reports("caller.ts", false), lines([18, 23, 28, 32, 36]));
    assert.deepEqual(reports("dropped.ts", false), lines([4]));
    assert.deepEqual(reports("final.ts", false), []);
    assert.deepEqual(reports("caller.ts", false), lines([18, 23, 28, 32, 36]));
    // Disk program nodes keep their starts; a longer callee and a trailing call do not.
    const divergent = readFileSync(resolve(root, "caller.ts"), "utf8").replace(
      "structural(asserted",
      "structuralX(asserted",
    );
    const typedConfig = {
      files: ["**/*.ts"],
      languageOptions: { parser: tsParser },
      plugins: { "no-mistakes": plugin },
      rules: { "no-mistakes/async-call-disposition": ["error", { targets }] },
    };
    assert.deepEqual(
      new Linter({ cwd: root })
        .verify(`${divergent}\nfoo();\n`, typedConfig, { filename: resolve(root, "caller.ts") })
        .map(({ line, messageId }) => [line, messageId]),
      lines([18, 23, 28, 32, 36]),
    );
    const missing = resolve(root, "not-in-program.ts");
    assert.deepEqual(
      new Linter({ cwd: root })
        .verify(
          readFileSync(resolve(root, "caller.ts"), "utf8"),
          {
            files: ["**/*.ts"],
            languageOptions: { parser: tsParser },
            plugins: { "no-mistakes": plugin },
            rules: { "no-mistakes/async-call-disposition": ["error", { targets }] },
          },
          { filename: missing },
        )
        .map(({ line, messageId }) => [line, messageId]),
      lines([6, 11, 14, 18, 23, 28, 32, 36]),
    );
    for (const directory of ["no-resolve", "invalid-config", "missing-extends"]) {
      assert.deepEqual(reports(resolve(proven, directory, "caller.ts")), [[3, "disposition"]]);
    }
    assert.deepEqual(
      new Linter({ cwd: resolve(__dirname, "..") })
        .verify(
          "enqueueEmail();\n",
          {
            files: ["**/*.js"],
            languageOptions: { parser: tsParser },
            plugins: { "no-mistakes": plugin },
            rules: { "no-mistakes/async-call-disposition": ["error", { targets }] },
          },
          { filename: resolve(__dirname, "../src/index.js") },
        )
        .map(({ line, messageId }) => [line, messageId]),
      [],
    );
  });
});

it("keeps today's reports under plain Oxlint", () => {
  assert.deepEqual(
    diagnostics(oxlint(false, ["caller.ts", "dropped.ts", "final.ts"])),
    [6, 11, 14, 18, 23, 28, 32, 36]
      .map((line) => ["caller.ts", line, "no-mistakes(async-call-disposition)"])
      .sort(),
  );
});

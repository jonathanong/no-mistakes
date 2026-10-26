import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { Linter } from "eslint";
import tsParser from "@typescript-eslint/parser";
import { it } from "vitest";
import { fixture } from "./async-promise-handoff-real.mjs";
import { __dirname, plugin } from "./helpers.mjs";

const root = resolve(fixture, "promise-handoff-alias");
const rules = JSON.parse(readFileSync(resolve(root, ".oxlintrc.json"), "utf8")).rules;
it.each([false, true])("resolves target aliases through tsconfig extends (typed=%s)", (typed) => {
  for (const [name, lines] of [
    ["caller.ts", [11, 15]],
    ["nested-caller.ts", [9, 13]],
    ["nested-consumers.ts", []],
    ["consumer.ts", [6]],
    ["controls.ts", [7, 8, 9, 10, 11]],
  ]) {
    const filename = resolve(root, name);
    const messages = new Linter({ cwd: root }).verify(
      readFileSync(filename, "utf8"),
      {
        files: ["**/*.ts"],
        languageOptions: {
          parser: tsParser,
          parserOptions: typed ? { project: resolve(root, "tsconfig.json") } : {},
        },
        plugins: { "no-mistakes": plugin },
        rules,
      },
      { filename },
    );
    assert.deepEqual(
      messages.map(({ line, messageId }) => [line, messageId]),
      lines.map((line) => [line, "disposition"]),
    );
  }
});

it("keeps source contracts conservative when optional proof peers are missing", () => {
  const result = spawnSync(
    process.execPath,
    [
      resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
      "--type-aware",
      "--config",
      resolve(root, ".oxlintrc.json"),
      "--format",
      "json",
      resolve(root, "caller.ts"),
      resolve(root, "consumer.ts"),
    ],
    {
      encoding: "utf8",
      env: { ...process.env, NODE_OPTIONS: `--require=${resolve(root, "without-peers.cjs")}` },
    },
  );
  assert.equal(result.status, 1, result.stderr);
  assert.deepEqual(
    JSON.parse(result.stdout)
      .diagnostics.map((item) => [
        item.filename.split("/").at(-1),
        item.labels[0].span.line,
        item.code,
      ])
      .sort(),
    [6, 11, 15].map((line) => ["caller.ts", line, "no-mistakes(async-call-disposition)"]).sort(),
  );
});

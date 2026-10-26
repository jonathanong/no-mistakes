import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { __dirname } from "./helpers.mjs";

export const root = resolve(
  __dirname,
  "../../../test-cases/eslint-plugin/async-call-disposition/fixture/handoff-data",
);
export const rules = JSON.parse(readFileSync(resolve(root, ".oxlintrc.json"), "utf8")).rules;
export const cases = {
  "accepted.ts": [],
  "controls.ts": [7, 9, 13, 17, 19, 22, 23, 24, 26, 30],
  "depth.ts": [37],
  "date-prototype.ts": [3, 4],
  "object-prototype.ts": [3, 4],
  "shadowed-date.ts": [3, 4],
  "array-prototype.ts": [3, 7],
};
export function realGate(typeAware) {
  const result = spawnSync(
    process.execPath,
    [
      resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
      ...(typeAware ? ["--type-aware"] : []),
      "--config",
      resolve(root, ".oxlintrc.json"),
      "--format",
      "json",
      ...Object.keys(cases).map((name) => resolve(root, name)),
    ],
    { encoding: "utf8" },
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
    Object.entries(cases)
      .flatMap(([name, lines]) =>
        lines.map((line) => [name, line, "no-mistakes(async-call-disposition)"]),
      )
      .sort(),
  );
}
if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  realGate(true);
  realGate(false);
  process.stdout.write("Real type-aware and plain Oxlint metadata gates passed\n");
}

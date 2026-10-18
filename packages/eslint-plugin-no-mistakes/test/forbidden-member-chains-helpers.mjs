import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { fixture, lint, __dirname } from "./helpers.mjs";

export const id = "forbidden-member-chains";
export const fixtureRoot = resolve(__dirname, "../../../test-cases/eslint-snippets/fixture", id);
// Keep this list identical to .oxlintrc.json.
export const chains = [
  { path: "records.entry", message: "Use the entry helper." },
  { path: "*.panel.open", message: "Open the panel through the helper." },
  { receiver: "*", path: ["drawer", "toggle"], message: "Toggle the drawer through the helper." },
  { path: "matrix.1", message: "Do not read the numeric slot." },
  { path: "*.#slot", message: "Do not read the private slot." },
];
export const entry = "Use the entry helper.";
export const panel = "Open the panel through the helper.";
export const drawer = "Toggle the drawer through the helper.";
export const slot = "Do not read the numeric slot.";
export const priv = "Do not read the private slot.";

export function check(name, options = { chains }) {
  return lint(
    fixture(`${id}/${name}`),
    { [`no-mistakes/${id}`]: ["error", options] },
    resolve(fixtureRoot, name),
  );
}

export function hits(messages) {
  return messages.map((message) => [
    message.line,
    message.column,
    message.endColumn,
    message.message,
  ]);
}

export function lineMessages(messages) {
  return hits(messages).map(([line, , , message]) => [line, message]);
}

export function oxlintFiles(names, config = ".oxlintrc.json") {
  const result = spawnSync(
    process.execPath,
    [
      resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
      "--config",
      resolve(fixtureRoot, config),
      "--format",
      "json",
      ...names.map((name) => resolve(fixtureRoot, name)),
    ],
    { encoding: "utf8" },
  );
  assert.equal(result.error, undefined, result.stderr);
  const report = JSON.parse(result.stdout || "{}");
  // Oxlint's default correctness rules warn on these expression-statement fixtures.
  // Parity is the configured member-chain rule, same as the other jsPlugins tests.
  const grouped = new Map(names.map((name) => [name, []]));
  for (const diagnostic of report.diagnostics || []) {
    if (!diagnostic.code?.includes(id)) continue;
    const name = diagnostic.filename?.split(/[\\/]/).pop();
    assert.ok(grouped.has(name), diagnostic.filename);
    grouped.get(name).push(diagnostic);
  }
  return grouped;
}

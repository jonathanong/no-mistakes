import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import tsParser from "@typescript-eslint/parser";
import { describe, expect, it } from "vitest";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { __dirname, fixture, plugin } from "./helpers.mjs";
const readRule = "no-consecutive-unbatched-reads";
const registrationRule = "require-top-level-registration";
const importedReads = {
  imports: [{ source: "@app/cache", imported: "cache" }],
  readMethods: ["get", "ttl"],
};
const localReads = { clients: ["client"], readMethods: ["get", "ttl"] };
const registration = {
  registrar: { source: "@app/lifecycle", imported: "register" },
  callback: "onStop",
};
const code = (name) => fixture(`client-contracts/${name}.ts`);
const languageOptions = {
  ecmaVersion: 2024,
  sourceType: "module",
  globals: {},
  parser: tsParser,
  parserOptions: { ecmaFeatures: { jsx: true } },
};
const diagnosticCase = (name, options) => ({
  code: code(name),
  filename: "fixture.ts",
  ...(options === undefined ? {} : { options: [options] }),
});
describe("configured client contracts", () => {
  it("snapshots read and registration diagnostics", async () => {
    await expect(
      snapshotDocument({
        [readRule]: snapshotSuite(readRule, languageOptions, {
          valid: [
            ...["reads-valid", "reads-shapes", "reads-wrong-import", "reads-shadowed"].map((name) =>
              diagnosticCase(name, importedReads),
            ),
            diagnosticCase("reads-invalid", { readMethods: ["get"] }),
            diagnosticCase("reads-invalid"),
            ...["file", "line", "next"].map((suffix) =>
              diagnosticCase(`reads-suppressed-${suffix}`, localReads),
            ),
          ],
          invalid: [
            diagnosticCase("reads-invalid", importedReads),
            diagnosticCase("reads-receiver-wrappers", importedReads),
            diagnosticCase("reads-edge", localReads),
            diagnosticCase("reads-invalid", { ...localReads, message: "Custom contract" }),
          ],
        }),
        [registrationRule]: snapshotSuite(registrationRule, languageOptions, {
          valid: [
            diagnosticCase("registration-string-import", registration),
            diagnosticCase("registration-valid", registration),
            diagnosticCase("registration-empty"),
            ...["file", "line", "next"].map((suffix) =>
              diagnosticCase(`registration-suppressed-${suffix}`, registration),
            ),
          ],
          invalid: [
            ...[
              "registration-invalid",
              "registration-type",
              "registration-type-specifier",
              "registration-wrong",
              "registration-local",
              "registration-unbound",
              "registration-empty",
            ].map((name) => diagnosticCase(name, registration)),
            diagnosticCase("registration-invalid", {
              ...registration,
              message: "Custom contract",
            }),
          ],
        }),
      }),
    ).toMatchFileSnapshot("snapshots/client-contracts.snap");
  });
  it("honors suppression directives through Oxlint", () => {
    const root = resolve(__dirname, "../../../test-cases/eslint-snippets/fixture/client-contracts");
    for (const kind of ["reads", "registration"]) {
      const result = spawnSync(
        process.execPath,
        [
          resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
          "--config",
          resolve(root, `.oxlintrc.${kind}.json`),
          "--format",
          "json",
          ...["file", "line", "next"].map((suffix) =>
            resolve(root, `${kind}-suppressed-${suffix}.ts`),
          ),
        ],
        { encoding: "utf8" },
      );
      assert.equal(result.error, undefined);
      assert.equal(result.status, 0, result.stdout + result.stderr);
      assert.deepEqual(
        JSON.parse(result.stdout).diagnostics.filter((item) => item.code.includes("no-mistakes/")),
        [],
      );
    }
  });
  it("keeps configured rules out of recommended presets", () => {
    assert.equal(plugin.configs.recommended.rules[`no-mistakes/${readRule}`], undefined);
    assert.equal(plugin.configs.recommended.rules[`no-mistakes/${registrationRule}`], undefined);
  });
  it("reports the same contracts through Oxlint", () => {
    const root = resolve(__dirname, "../../../test-cases/eslint-snippets/fixture/client-contracts");
    const result = spawnSync(
      process.execPath,
      [
        resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
        "--config",
        resolve(root, ".oxlintrc.json"),
        "--format",
        "json",
        resolve(root, "reads-invalid.ts"),
        resolve(root, "reads-receiver-wrappers.ts"),
        resolve(root, "registration-invalid.ts"),
        resolve(root, "registration-valid.ts"),
      ],
      { encoding: "utf8" },
    );
    assert.equal(result.error, undefined);
    assert.equal(result.status, 1, result.stderr);
    const diagnostics = JSON.parse(result.stdout).diagnostics;
    assert.equal(diagnostics.filter((item) => item.code.includes(readRule)).length, 6);
    assert.equal(diagnostics.filter((item) => item.code.includes(registrationRule)).length, 3);
  });
});

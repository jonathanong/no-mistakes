import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { describe, it } from "vitest";
import { __dirname, fixture, lint, messages, plugin } from "./helpers.mjs";
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
describe("configured client contracts", () => {
  it("reports adjacent same-client reads through aliased runtime imports", () => {
    assert.deepEqual(
      messages(code("reads-invalid"), readRule, importedReads, "fixture.ts"),
      Array(3).fill("unbatchedReads"),
    );
  });
  it("unwraps transparent TypeScript receiver wrappers", () => {
    assert.deepEqual(
      messages(code("reads-receiver-wrappers"), readRule, importedReads, "fixture.ts"),
      Array(3).fill("unbatchedReads"),
    );
  });
  it("preserves result dependencies, batching, scope, and method exclusions", () => {
    for (const name of ["reads-valid", "reads-shapes", "reads-wrong-import", "reads-shadowed"])
      assert.deepEqual(messages(code(name), readRule, importedReads, "fixture.ts"), [], name);
    assert.equal(messages(code("reads-edge"), readRule, localReads, "fixture.ts").length, 2);
    assert.deepEqual(
      messages(code("reads-invalid"), readRule, { readMethods: ["get"] }, "fixture.ts"),
      [],
    );
    assert.deepEqual(messages(code("reads-invalid"), readRule, undefined, "fixture.ts"), []);
  });
  it("requires a genuine runtime registration directly at module top level", () => {
    assert.deepEqual(
      messages(code("registration-string-import"), registrationRule, registration, "fixture.ts"),
      [],
    );
    assert.deepEqual(
      messages(code("registration-valid"), registrationRule, registration, "fixture.ts"),
      [],
    );
    for (const name of [
      "registration-invalid",
      "registration-type",
      "registration-type-specifier",
      "registration-wrong",
      "registration-local",
      "registration-unbound",
      "registration-empty",
    ])
      assert.deepEqual(
        messages(code(name), registrationRule, registration, "fixture.ts"),
        ["registration"],
        name,
      );
    assert.deepEqual(
      messages(code("registration-empty"), registrationRule, undefined, "fixture.ts"),
      [],
    );
  });
  it("supports custom diagnostics and all suppression forms", () => {
    for (const [kind, rule, options] of [
      ["reads", readRule, localReads],
      ["registration", registrationRule, registration],
    ]) {
      for (const suffix of ["file", "line", "next"])
        assert.deepEqual(
          messages(code(`${kind}-suppressed-${suffix}`), rule, options, "fixture.ts"),
          [],
        );
      const invalid = kind === "reads" ? "reads-invalid" : "registration-invalid";
      const results = lint(
        code(invalid),
        { [`no-mistakes/${rule}`]: ["error", { ...options, message: "Custom contract" }] },
        "fixture.ts",
      );
      assert.ok(results.every((result) => result.message === "Custom contract"));
    }
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

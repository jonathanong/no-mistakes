const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const { dirname, join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const addon = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH;

for (const config of ["default", "ts", "blanket", "custom", "none", "excluded", "quoted"]) {
  test(
    `compiled forbidden lint suppression ${config} preserves CLI, async and batched parity`,
    { skip: !addon?.endsWith(".node") },
    async () => {
      const cjs = require("../../index.js");
      const esm = await import("../../index.mjs");
      const root = join(
        __dirname,
        "../../../../test-cases/rules/forbidden-lint-suppressions/fixture",
      );
      const cli =
        process.env.NO_MISTAKES_TEST_CLI_PATH ||
        join(dirname(addon), process.platform === "win32" ? "no-mistakes.exe" : "no-mistakes");
      const options = { root, config: join(root, `${config}.yml`) };
      const pending = cjs.check(options);
      assert.ok(pending instanceof Promise);
      const report = await pending;
      assert.deepEqual(await esm.check(options), report);
      const request = { ...options, reports: [{ type: "check" }] };
      assert.deepEqual((await cjs.analyzeProject(request)).reports[0].result, report);
      assert.deepEqual((await esm.analyzeProject(request)).reports[0].result, report);
      const findings = report.rules.filter((row) => row.rule === "forbidden-lint-suppressions");
      assert.deepEqual(report.warnings, []);
      assert.deepEqual(report.advisories, []);
      assert.ok(findings.every((row) => row.file.startsWith("src/") && row.line > 0));
      if (config === "ts") {
        assert.deepEqual(
          findings
            .filter((row) => row.file === "src/directives.tsx")
            .map((row) => [row.line, row.target]),
          [
            [9, "@ts-ignore"],
            [11, "@ts-expect-error"],
          ],
        );
        assert.deepEqual(
          findings
            .filter((row) => row.file === "src/ts-blocks.ts")
            .map((row) => [row.line, row.target]),
          [
            [1, "@ts-ignore"],
            [4, "@ts-ignore"],
            [7, "@ts-expect-error"],
            [13, "@ts-expect-error"],
          ],
        );
      } else if (config === "none" || config === "excluded") {
        assert.deepEqual(findings, []);
      } else {
        assert.deepEqual(
          findings
            .filter((row) => row.file === "src/directives.tsx" && row.target !== "*")
            .map((row) => [row.line, row.target]),
          config === "quoted"
            ? []
            : [
                [1, "react/purity"],
                [2, "react/purity"],
                [3, "react-hooks-js/refs"],
                [3, "react/purity"],
                [4, "react/purity"],
                [5, "react-hooks-js/purity"],
                [6, "react/purity"],
                [20, "react-hooks-js/refs"],
              ],
        );
        assert.deepEqual(
          findings
            .filter((row) => row.file === "src/directives.tsx" && row.target === "*")
            .map((row) => row.line),
          config === "blanket" ? [7, 8] : [],
        );
        assert.deepEqual(
          findings
            .filter((row) => row.file === "src/quoted.ts")
            .map((row) => [row.line, row.target]),
          [
            [1, "react-hooks-js/refs"],
            [1, "react/purity"],
            [2, "react-hooks-js/refs"],
            [2, "react/purity"],
            ...(config === "quoted" ? [[4, "custom/no--bad"]] : []),
            [5, "react/purity"],
            [6, "react/purity"],
          ],
        );
        if (config === "custom") {
          assert.ok(findings.every((row) => row.message.endsWith("Keep compiler checks enabled.")));
        }
      }
      const child = spawnSync(
        cli,
        ["check", "--root", root, "--config", options.config, "--format", "json"],
        { encoding: "utf8" },
      );
      assert.equal(child.error, undefined);
      assert.equal(child.status, findings.length ? 1 : 0, child.stderr);
      assert.deepEqual(JSON.parse(child.stdout), report);
    },
  );
}

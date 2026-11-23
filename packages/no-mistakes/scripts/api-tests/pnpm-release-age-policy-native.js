const { spawnSync } = require("node:child_process");
const { dirname, join } = require("node:path");
const { assert, test, repositoryRoot } = require("./context");

const addon = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH;
const rule = "pnpm-release-age-policy";
const fixtures = join(repositoryRoot, "test-cases/rules", rule);

for (const [fixture, expected] of [
  ["glob-pass", []],
  ["glob-dependabot-mismatch", [[".github/dependabot.yml", "acme-cli-macos"]]],
]) {
  test(
    `compiled pnpm release-age glob policy agrees across CLI and Node for ${fixture}`,
    { skip: !addon?.endsWith(".node"), timeout: 20_000 },
    async () => {
      const api = require("../../index.js");
      const esm = await import("../../index.mjs");
      const options = {
        root: join(fixtures, "fixture", fixture),
        config: join(fixtures, "configs", "registered-glob.yml"),
      };
      const pending = api.check(options);
      assert.ok(pending instanceof Promise);
      const report = await pending;
      assert.deepEqual(
        report.rules.map(({ rule: id, file, target }) => [id, file, target]),
        expected.map(([file, target]) => [rule, file, target]),
      );
      assert.deepEqual(await esm.check(options), report);
      const request = { ...options, reports: [{ type: "check" }] };
      const batch = await api.analyzeProject(request);
      assert.deepEqual(batch.reports[0].result, report);
      assert.deepEqual(await esm.analyzeProject(request), batch);

      const cli =
        process.env.NO_MISTAKES_TEST_CLI_PATH ||
        join(dirname(addon), process.platform === "win32" ? "no-mistakes.exe" : "no-mistakes");
      const child = spawnSync(
        cli,
        [
          "check",
          "--lock-timeout",
          "0",
          "--root",
          options.root,
          "--config",
          options.config,
          "--json",
        ],
        { encoding: "utf8" },
      );
      assert.equal(child.error, undefined);
      assert.equal(child.status, expected.length ? 1 : 0, child.stderr);
      assert.deepEqual(JSON.parse(child.stdout), report);
    },
  );
}

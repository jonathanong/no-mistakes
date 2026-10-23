const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const { join } = require("node:path");

async function main() {
  const cjs = require("../../../../../packages/no-mistakes/index.js");
  const esm = await import("../../../../../packages/no-mistakes/index.mjs");
  const policies = ["report", "advise", "ignore"];
  for (const policy of policies) {
    const options = { root: __dirname, config: join(__dirname, `${policy}.yml`) };
    const pending = cjs.check(options);
    assert.ok(pending instanceof Promise);
    const report = await pending;
    const findings = report.rules.filter((row) => row.rule === "shellcheck-runner");
    const advisories = report.advisories.filter((row) => row.rule === "shellcheck-runner");
    assert.equal(findings.length > 0, policy === "report");
    assert.equal(advisories.length > 0, policy === "advise");
    for (const row of [...findings, ...advisories]) {
      assert.equal(row.target, "unanalyzable");
      assert.match(row.message, /shellcheck/i);
    }
    assert.deepEqual(await esm.check(options), report);
    const request = { ...options, reports: [{ type: "check" }] };
    const batch = await cjs.analyzeProject(request);
    assert.deepEqual(batch.reports[0].result, report);
    assert.deepEqual(await esm.analyzeProject(request), batch);
    const cli = spawnSync(
      process.argv[2],
      ["check", "--root", __dirname, "--config", options.config, "--format", "json"],
      { encoding: "utf8" },
    );
    assert.equal(cli.error, undefined);
    assert.equal(cli.status, policy === "report" ? 1 : 0, cli.stderr);
    assert.deepEqual(JSON.parse(cli.stdout), report);
  }
  process.stdout.write(JSON.stringify(policies));
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});

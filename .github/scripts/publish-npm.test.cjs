const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const path = require("node:path");
const { test } = require("node:test");
const { VISIBILITY_WINDOW_MS, distTag, publish } = require("./publish-npm.cjs");
const root = path.resolve(__dirname, "../..");
const scenarios = JSON.parse(readFileSync(path.join(root, "test-cases/npm-publish/retry-and-tags/fixture/scenarios.json"), "utf8"));

for (const scenario of scenarios) {
  test(scenario.name, async () => {
    const results = [...scenario.results];
    const packageName = scenario.packageName || "eslint-plugin-no-mistakes";
    const tags = [];
    const delays = [];
    let elapsed = 0;
    const options = {
      now: () => elapsed,
      run(args, timeoutMs) {
        assert.ok(results.length, `Unexpected npm command: ${args.join(" ")}`);
        const result = results.shift();
        if (result.timeoutMs !== undefined) assert.equal(timeoutMs, result.timeoutMs);
        elapsed += result.elapsedMs || 0;
        const requests = {
          version: ["view", `${packageName}@0.109.8`, "version", "--json"],
          latest: ["view", packageName, "dist-tags.latest", "--json"],
          publish: ["publish", `./packages/${packageName}`, "--provenance", "--access", "public", "--tag", scenario.tags[tags.length]],
        };
        assert.deepEqual(args, requests[result.request]);
        if (args[0] === "publish") tags.push(args[6]);
        if (result.throw) throw Object.assign(new Error(result.throw.message), result.throw);
        return result;
      },
      async sleep(ms) { delays.push(ms); elapsed += ms; },
      windowMs: scenario.windowMs === "default" ? undefined : (scenario.windowMs ?? 120000),
    };
    const result = publish(packageName, "0.109.8", options);
    if (scenario.error) await assert.rejects(result, new RegExp(scenario.error));
    else await result;
    assert.equal(results.length, 0);
    assert.deepEqual(tags, scenario.tags);
    assert.deepEqual(delays, scenario.delays || []);
  });
}

for (const [version, latest, tag] of [
  ["0.109.8", "0.109.9", "backfill"],
  ["0.110.0", "0.109.9", "latest"],
  ["0.10.0", "0.9.9", "latest"],
  ["1.0.0", "0.999.999", "latest"],
  ["0.109.8", "0.109.8", "latest"],
  ["1.0.0-rc.1", "0.109.9", "backfill"],
  ["1.0.0", "1.1.0-rc.1", "backfill"],
  ["1.0.0+build", "1.1.0", "backfill"],
]) test(`tag for ${version} after ${latest}`, () => assert.equal(distTag(version, latest), tag));

const workflow = readFileSync(path.join(root, ".github/workflows/release.yml"), "utf8");
const job = (name) => workflow.split(`\n  ${name}:\n`)[1].split(/\n  [\w-]+:\n/)[0];
test("plugin publication requires fetchable native packages and root", () => {
  const needs = (name) => job(name).split("    needs:\n")[1].split("    runs-on:")[0];
  assert.match(needs("publish-js"), /      - verify-npm-platform\n/);
  assert.doesNotMatch(needs("publish-js"), /publish-eslint/);
  assert.match(needs("verify-npm-root"), /      - publish-js\n/);
  assert.match(needs("publish-eslint"), /      - verify-npm-root\n/);
  for (const name of ["publish-js", "verify-npm-root", "publish-eslint"]) {
    assert.doesNotMatch(job(name), /if:.*always\(\)/);
  }
});
test("every npm channel uses workflow-owned publish tooling, including older tags", () => {
  for (const name of ["publish", "publish-js", "publish-eslint"]) {
    const block = job(name);
    assert.match(block, /ref: \$\{\{ github.workflow_sha \}\}/);
    assert.match(block, /persist-credentials: false/);
    assert.match(block, /mv -- publish-tools "\$RUNNER_TEMP\/publish-tools"/);
    assert.match(block, /node "\$RUNNER_TEMP\/publish-tools\/\.github\/scripts\/publish-npm.cjs" "\$pkg" "\$version"/);
    assert.doesNotMatch(block, /npm publish /);
    assert.match(block, /--timeout-ms 1800000/);
  }
});
test("npm publish steps outlast the visibility window for every package", () => {
  const windowMinutes = VISIBILITY_WINDOW_MS / 60000;
  assert.equal(windowMinutes, 15);
  for (const name of ["publish", "publish-js", "publish-eslint"]) {
    const block = job(name);
    const step = block.split(/\n      - name: Publish npm [^\n]*\n/)[1] ?? block.split(/\n      - name: Publish ESLint package with OIDC\n/)[1];
    const stepMinutes = Number(step.match(/timeout-minutes: (\d+)/)[1]);
    const jobMinutes = Number(block.match(/\n    timeout-minutes: (\d+)/)[1]);
    const packages = step.split("packages=(\n")[1].split(")")[0].trim().split(/\s+/).length;
    assert.ok(stepMinutes >= packages * (windowMinutes + 2) + 5, `${name} step needs ${packages} x ${windowMinutes + 2} minutes`);
    assert.ok(jobMinutes >= stepMinutes + 10, `${name} job must leave setup time around the publish step`);
  }
});

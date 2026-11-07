const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const path = require("node:path");
const { test } = require("node:test");
const { distTag, publish } = require("./publish-npm.cjs");
const root = path.resolve(__dirname, "../..");
const scenarios = JSON.parse(readFileSync(path.join(root, "test-cases/npm-publish/retry-and-tags/fixture/scenarios.json"), "utf8"));

for (const scenario of scenarios) {
  test(scenario.name, async () => {
    const results = [...scenario.results];
    const tags = [];
    const delays = [];
    const options = {
      run(args) {
        assert.ok(results.length, `Unexpected npm command: ${args.join(" ")}`);
        const result = results.shift();
        const requests = {
          version: ["view", "eslint-plugin-no-mistakes@0.109.8", "version", "--json"],
          latest: ["view", "eslint-plugin-no-mistakes", "dist-tags.latest", "--json"],
          publish: ["publish", "./packages/eslint-plugin-no-mistakes", "--provenance", "--access", "public", "--tag", scenario.tags[tags.length]],
        };
        assert.deepEqual(args, requests[result.request]);
        if (args[0] === "publish") tags.push(args[6]);
        if (result.throw) throw Object.assign(new Error(result.throw.message), result.throw);
        return result;
      },
      async sleep(ms) { delays.push(ms); },
    };
    const result = publish("eslint-plugin-no-mistakes", "0.109.8", options);
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
test("root publication requires successful plugin publication", () => {
  const needs = job("publish-js").split("    needs:\n")[1].split("    runs-on:")[0];
  assert.match(needs, /      - publish-eslint\n/);
  assert.doesNotMatch(job("publish-js"), /if:.*always\(\)/);
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

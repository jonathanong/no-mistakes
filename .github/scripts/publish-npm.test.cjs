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
const publishScript = readFileSync(path.join(root, ".github/scripts/publish-one-npm.sh"), "utf8");
const job = (name) => {
  const parts = workflow.split(`\n  ${name}:\n`);
  assert.equal(parts.length, 2, name);
  return parts[1].split(/\n  [\w-]+:\n/)[0];
};
test("plugin publication requires fetchable native packages and root", () => {
  const needs = (name) => `${job(name).split("    needs:\n")[1].split(/\n    [a-z]/)[0]}\n`;
  for (const verify of ["verify-npm-darwin", "verify-npm-linux-x64", "verify-npm-linux-arm", "verify-npm-windows"]) {
    assert.match(needs("publish-js"), new RegExp(`      - ${verify}\\n`));
  }
  assert.doesNotMatch(needs("publish-js"), /publish-eslint|verify-npm-root/);
  for (const verify of ["verify-npm-root-linux-x64", "verify-npm-root-linux-arm", "verify-npm-root-windows"]) {
    assert.match(needs(verify), /      - publish-js\n/);
    assert.match(needs("publish-eslint"), new RegExp(`      - ${verify}\\n`));
  }
  assert.match(needs("verify-npm-root-darwin"), /      - publish-js\n/);
  assert.doesNotMatch(needs("publish-eslint"), /verify-npm-root-darwin/);
  assert.match(needs("release-complete"), /      - publish-eslint\n/);
  assert.match(needs("release-complete"), /      - verify-npm-root-darwin\n/);
  for (const name of ["publish-js", "publish-eslint", "verify-npm-root-darwin", "verify-npm-linux-x64"]) {
    assert.doesNotMatch(job(name), /if:.*always\(\)/);
  }
});
test("every npm channel uses workflow-owned publish tooling, including older tags", () => {
  for (const name of ["publish-native-linux-x64", "publish-js", "publish-eslint"]) {
    const block = job(name);
    assert.match(block, /ref: \$\{\{ github.workflow_sha \}\}/);
    assert.match(block, /persist-credentials: false/);
    assert.match(block, /mv -- publish-tools "\$RUNNER_TEMP\/publish-tools"/);
    assert.match(block, /bash "\$RUNNER_TEMP\/publish-tools\/\.github\/scripts\/publish-one-npm\.sh"/);
    if (name.startsWith("publish-native-")) {
      assert.match(block, /bash "\$RUNNER_TEMP\/publish-tools\/\.github\/scripts\/stage-one-platform\.sh"/);
    }
    assert.doesNotMatch(block, /npm publish /);
  }
  for (const name of ["publish-native-linux-arm", "publish-native-darwin", "publish-native-windows"]) {
    const block = job(name);
    assert.match(block, /ref: \$\{\{ github.workflow_sha \}\}/);
    assert.match(block, /bash "\$RUNNER_TEMP\/publish-tools\/\.github\/scripts\/publish-one-npm\.sh"/);
    assert.match(block, /bash "\$RUNNER_TEMP\/publish-tools\/\.github\/scripts\/stage-one-platform\.sh"/);
    assert.match(block, /actions\/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7\.0\.1/);
  }
  const publishAt = publishScript.indexOf('publish-npm.cjs" "$package" "$version"');
  const waitAt = publishScript.indexOf('--version "$version" --timeout-ms 1800000');
  assert.ok(publishAt >= 0 && waitAt > publishAt);
});
test("npm publish steps outlast the visibility window for every package", () => {
  const windowMinutes = VISIBILITY_WINDOW_MS / 60000;
  assert.equal(windowMinutes, 15);
  const jobs = [
    "publish-native-linux-x64",
    "publish-native-linux-arm",
    "publish-native-darwin",
    "publish-native-windows",
    "publish-js",
    "publish-eslint",
  ];
  // timeout-ms 0 still spends one 60s request. Each of three publish attempts
  // can spend two minutes on version, latest, and npm publish before the
  // 15 minute packument window and the 30 minute tarball wait.
  const probeMinutes = 1;
  const attemptMinutes = 3 * 2;
  const overheadMinutes = probeMinutes + 3 * attemptMinutes;
  assert.equal(publishScript.includes("--timeout-ms 0"), true);
  assert.equal(publishScript.includes("--timeout-ms 1800000"), true);
  for (const name of jobs) {
    const block = job(name);
    const step = block.split(/\n      - name: Publish [^\n]*\n/)[1];
    const stepMinutes = Number(step.match(/timeout-minutes: (\d+)/)[1]);
    const jobMinutes = Number(block.match(/\n    timeout-minutes: (\d+)/)[1]);
    const tarballMinutes = 30;
    assert.ok(
      stepMinutes >= windowMinutes + tarballMinutes + overheadMinutes,
      `${name} step needs the probe, three publish attempts, the packument window, and the tarball wait`,
    );
    assert.ok(jobMinutes >= stepMinutes + 10, `${name} job leaves ten minutes for setup`);
    const stepTimeouts = block.split(/\n    steps:[^\n]*\n/)[1].split(/\n?      - /).slice(1).map((s) => Number(s.match(/\n        timeout-minutes: (\d+)/)?.[1]));
    assert.ok(stepTimeouts.every(Number.isInteger), `${name} steps all need literal timeouts`);
    const stepSum = stepTimeouts.reduce((a, b) => a + b, 0);
    assert.ok(jobMinutes >= Math.max(...stepTimeouts), `${name} job must cover its longest step`);
    assert.ok(jobMinutes < stepSum, `${name} job must stay below the sum of its step timeouts (${stepSum})`);
  }
});

const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const { readFileSync } = require("node:fs");
const path = require("node:path");
const { test } = require("node:test");
const { distTag, publish } = require("./publish-npm.cjs");
const { readWorkflow } = require("./workflow-contract.cjs");
const root = path.resolve(__dirname, "../..");
const scenarios = JSON.parse(readFileSync(path.join(root, "test-cases/npm-publish/retry-and-tags/fixture/scenarios.json"), "utf8"));

for (const scenario of scenarios) {
  test(scenario.name, async () => {
    const results = [...scenario.results];
    const packageName = scenario.packageName || "eslint-plugin-no-mistakes";
    const tags = [];
    const delays = [];
    const options = {
      run(args, timeoutMs) {
        assert.ok(results.length, `Unexpected npm command: ${args.join(" ")}`);
        const result = results.shift();
        if (result.timeoutMs !== undefined) assert.equal(timeoutMs, result.timeoutMs);
        const requests = {
          version: ["view", `${packageName}@0.109.8`, "version", "--json", "--prefer-online"],
          latest: ["view", packageName, "dist-tags.latest", "--json", "--prefer-online"],
          publish: ["publish", `./packages/${packageName}`, "--provenance", "--access", "public", "--tag", scenario.tags[tags.length]],
        };
        assert.deepEqual(args, requests[result.request]);
        if (args[0] === "publish") tags.push(args[6]);
        if (result.throw) throw Object.assign(new Error(result.throw.message), result.throw);
        return result;
      },
      async sleep(ms) { delays.push(ms); },
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

const { jobs } = readWorkflow(path.join(root, ".github/workflows/release.yml"));
const publishScript = path.join(root, ".github/scripts/publish-one-npm.sh");
const npmFixture = path.join(root, "test-cases/npm-publish/publish-wrapper/fixture");
const platforms = ["linux-x64", "linux-arm", "darwin", "windows"];
const platformPublishers = platforms.map((target) => `publish-native-${target}`);

test("root publication requires all four platform install and load jobs", () => {
  assert.deepEqual(new Set(jobs["publish-js"].needs), new Set([
    "prepare",
    ...platforms.map((target) => `verify-npm-${target}`),
  ]));
  assert.equal(jobs["publish-js"].if, undefined);
  for (const target of platforms) {
    const job = jobs[`verify-npm-${target}`];
    assert.deepEqual(job.needs, ["prepare", `publish-native-${target}`]);
    assert.equal(job.if, undefined);
    const smoke = job.steps.find((step) => step.name === "Install and load published package");
    assert.match(smoke.run, /npm install/);
    assert.match(smoke.run, /smoke-published-native\.js/);
  }
});

test("ESLint publication and release completion require root verification", () => {
  for (const target of ["linux-x64", "linux-arm", "windows"]) {
    const verify = `verify-npm-root-${target}`;
    assert.ok(jobs[verify].needs.includes("publish-js"));
    assert.ok(jobs["publish-eslint"].needs.includes(verify));
  }
  assert.ok(jobs["verify-npm-root-darwin"].needs.includes("publish-js"));
  assert.ok(!jobs["publish-eslint"].needs.includes("verify-npm-root-darwin"));
  assert.ok(jobs["release-complete"].needs.includes("publish-eslint"));
  assert.ok(jobs["release-complete"].needs.includes("verify-npm-root-darwin"));
  assert.equal(jobs["publish-eslint"].if, undefined);
});

test("platform publishers stage artifacts without installing or building the workspace", () => {
  for (const name of platformPublishers) {
    const job = jobs[name];
    assert.deepEqual(job.needs, ["prepare", "validate", name.replace("publish-", "build-")]);
    const commands = job.steps.map(({ run }) => run).filter(Boolean);
    const stage = commands.findIndex((run) => run.includes("stage-one-platform.sh"));
    const publishAt = commands.findIndex((run) => run.includes("publish-one-npm.sh"));
    assert.ok(stage >= 0 && publishAt > stage, name);
    assert.ok(commands.every((run) => !/pnpm (?:install|run build:js)/.test(run)), name);
  }
});

test("npm publishers use workflow-owned tooling and allow for upload retries and tarball verification", () => {
  // Two failed attempts can use four two-minute npm commands each. An
  // accepted attempt can use three commands, plus 30 seconds of retry delays.
  const publishMinutes = 2 * 4 * 2 + 3 * 2 + 0.5;
  const tarballMinutes = 45;
  for (const name of [...platformPublishers, "publish-js", "publish-eslint"]) {
    const job = jobs[name];
    const tooling = job.steps.find((step) => step.name === "Checkout publish tooling");
    assert.match(tooling.uses, /^actions\/checkout@[a-f0-9]{40}$/);
    assert.equal(tooling.with.ref, "${{ github.workflow_sha }}");
    assert.equal(tooling.with["persist-credentials"], false);
    assert.equal(tooling.with.path, "publish-tools");
    assert.ok(job.steps.some(({ run }) => run === 'mv -- publish-tools "$RUNNER_TEMP/publish-tools"'));
    const publishStep = job.steps.find(({ run }) => run?.includes("publish-one-npm.sh"));
    assert.match(publishStep.run, /bash "\$RUNNER_TEMP\/publish-tools\/\.github\/scripts\/publish-one-npm\.sh"/);
    assert.ok(publishStep["timeout-minutes"] >= publishMinutes + tarballMinutes, name);

    const stepTimeouts = job.steps.map((step) => step["timeout-minutes"]);
    assert.ok(stepTimeouts.every(Number.isInteger), name);
    assert.ok(job["timeout-minutes"] >= publishStep["timeout-minutes"] + 10, name);
    assert.ok(job["timeout-minutes"] < stepTimeouts.reduce((sum, minutes) => sum + minutes, 0), name);
  }
});

test("Windows keeps optimized CLI and addon builds in the release workflow", () => {
  const windows = jobs["build-native-windows"];
  assert.equal(windows.env.CARGO_PROFILE_RELEASE_LTO, "false");
  assert.equal(windows.env.CARGO_PROFILE_RELEASE_CODEGEN_UNITS, "16");
  assert.equal(jobs["build-native-darwin"].env.CARGO_PROFILE_RELEASE_LTO, "thin");
  assert.ok(windows.steps.some(({ run }) => run?.includes("cargo-release-build.sh")));
  const buildScript = readFileSync(path.join(root, ".github/scripts/cargo-release-build.sh"), "utf8");
  assert.match(buildScript, /NO_MISTAKES_BUILD_NAPI=1 cargo build --release --locked/);
});

for (const { name, publishStatus, verifyStatus, attempts, status } of [
  { name: "checks tarball once after npm accepts or skips upload", publishStatus: 0, verifyStatus: 0, attempts: ["publish", "verify"], status: 0 },
  { name: "fails when the verified tarball is unavailable", publishStatus: 0, verifyStatus: 1, attempts: ["publish", "verify"], status: 1 },
  { name: "skips tarball verification when publishing fails", publishStatus: 17, verifyStatus: 0, attempts: ["publish"], status: 17 },
]) {
  test(name, () => {
    const result = spawnSync("bash", [publishScript, "0.109.8", "no-mistakes"], {
      cwd: npmFixture,
      encoding: "utf8",
      env: {
        ...process.env,
        PATH: `${npmFixture}${path.delimiter}${process.env.PATH}`,
        PUBLISH_TOOLS: "/publish-tools",
        TEST_PUBLISH_STATUS: String(publishStatus),
        TEST_VERIFY_STATUS: String(verifyStatus),
      },
    });
    assert.ifError(result.error);
    assert.equal(result.status, status, result.stderr);
    assert.deepEqual(result.stdout.trim().split("\n"), attempts);
    if (verifyStatus) assert.match(result.stderr, /no-mistakes@0\.109\.8 tarball verification failed/);
  });
}

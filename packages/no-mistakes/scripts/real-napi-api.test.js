const assert = require("node:assert/strict");
const { fork } = require("node:child_process");
const { once } = require("node:events");
const { readFileSync } = require("node:fs");
const { copyFile, cp, mkdtemp, mkdir, rm, stat, utimes } = require("node:fs/promises");
const { tmpdir } = require("node:os");
const { join, resolve } = require("node:path");
const { setTimeout: delay } = require("node:timers/promises");
const { Worker } = require("node:worker_threads");
const test = globalThis.test || require("node:test").test;

const repositoryRoot = join(__dirname, "..", "..", "..");
const fixtureRoot = join(repositoryRoot, "fixtures", "napi", "real-addon-dependencies");
const selectorGroupingFixtureRoot = join(
  repositoryRoot,
  "fixtures",
  "test-plan",
  "test-runner-selector-grouping",
);
const lockHolderFixture = join(
  repositoryRoot,
  "fixtures",
  "napi",
  "planning-artifact-lock",
  "hold.js",
);
const workerLockHolderFixture = join(
  repositoryRoot,
  "fixtures",
  "napi",
  "planning-artifact-lock",
  "worker.js",
);
const expectedReport = JSON.parse(readFileSync(join(fixtureRoot, "expected.json"), "utf8"));
const freshTopologyFixture = join(repositoryRoot, "fixtures", "napi", "fresh-topology");
const addonPath = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH;
const compiledAddonPath = addonPath && addonPath.endsWith(".node") ? addonPath : undefined;

test(
  "compiled async N-API dependencies API matches the CLI fixture contract",
  { skip: !compiledAddonPath, timeout: 20_000 },
  async () => {
    assert.equal(resolve(addonPath), addonPath);
    assert.match(addonPath, /\.node$/);

    const api = require("../index.js");
    const pendingReport = api.dependencies({
      root: fixtureRoot,
      files: ["entry.ts"],
      relationships: ["import"],
    });

    assert.equal(typeof pendingReport.then, "function");
    assert.deepEqual(await pendingReport, expectedReport);
  },
);

function workflowPaths(topology) {
  return topology.workflows.map((workflow) => workflow.path).sort();
}

function jobIds(topology) {
  return topology.jobs.map((job) => job.id).sort();
}

async function batchedTopology(api, options) {
  const result = await api.analyzeProject({
    ...options,
    reports: [{ type: "ciTopology" }],
  });
  return result.reports[0].result;
}

test(
  "compiled topology calls read workflow and inherited config changes from the current filesystem",
  { skip: !compiledAddonPath, timeout: 20_000 },
  async () => {
    const directory = await mkdtemp(join(tmpdir(), "no-mistakes-fresh-topology-"));
    const root = join(directory, "project");
    const workflows = join(root, ".github", "workflows");
    const inheritedConfig = join(root, "configs", "inherited.yml");
    try {
      await cp(join(freshTopologyFixture, "project"), root, { recursive: true });
      const api = require("../index.js");

      assert.deepEqual(workflowPaths(await api.ciTopology({ root })), [
        ".github/workflows/initial.yml",
      ]);

      await copyFile(
        join(freshTopologyFixture, "changes", "workflow-edited.yml"),
        join(workflows, "initial.yml"),
      );
      assert.deepEqual(jobIds(await api.ciTopology({ root })), [
        ".github/workflows/initial.yml#edited",
      ]);

      await copyFile(
        join(freshTopologyFixture, "changes", "workflow-added.yml"),
        join(workflows, "added.yml"),
      );
      assert.deepEqual(workflowPaths(await api.ciTopology({ root })), [
        ".github/workflows/added.yml",
        ".github/workflows/initial.yml",
      ]);

      await rm(join(workflows, "initial.yml"));
      assert.deepEqual(workflowPaths(await api.ciTopology({ root })), [
        ".github/workflows/added.yml",
      ]);

      const originalConfigStat = await stat(inheritedConfig);
      const inheritedOptions = { root, config: "configs/inherited.yml" };
      const firstInherited = await api.ciTopology(inheritedOptions);
      assert.deepEqual(workflowPaths(firstInherited), ["inherited/first/first.yml"]);
      assert.deepEqual(await batchedTopology(api, inheritedOptions), firstInherited);

      // Keep the timestamp stable: freshness must not depend on config metadata.
      await copyFile(
        join(freshTopologyFixture, "changes", "inherited-config-updated.yml"),
        inheritedConfig,
      );
      await utimes(inheritedConfig, originalConfigStat.atime, originalConfigStat.mtime);
      const updatedInherited = await api.ciTopology(inheritedOptions);
      assert.deepEqual(workflowPaths(updatedInherited), ["inherited/second/second.yml"]);
      assert.deepEqual(await batchedTopology(api, inheritedOptions), updatedInherited);
    } finally {
      await rm(directory, { recursive: true, force: true });
    }
  },
);

test(
  "compiled async N-API test plans preserve runner selectors and match analyzeProject",
  { skip: !compiledAddonPath, timeout: 20_000 },
  async () => {
    const root = join(selectorGroupingFixtureRoot, "cargo");
    const options = {
      root,
      framework: "cargo",
      environment: "all",
      changedFiles: ["app/src/lib.rs"],
    };
    const direct = await require("../index.js").testsPlan(options);
    const batched = await require("../index.js").analyzeProject({
      root,
      reports: [
        {
          type: "testsPlan",
          framework: options.framework,
          environment: options.environment,
          changedFiles: options.changedFiles,
        },
      ],
    });

    assert.deepEqual(batched.reports[0].result, direct);
    assert.deepEqual(
      direct.executionTargets.map(({ runnerArgs, testFiles }) => ({ runnerArgs, testFiles })),
      [
        { runnerArgs: ["-p", "app", "--test", "a"], testFiles: ["app/tests/a.rs"] },
        { runnerArgs: ["-p", "app", "--test", "b"], testFiles: ["app/tests/b.rs"] },
      ],
    );

    const swiftOptions = {
      root: selectorGroupingFixtureRoot,
      config: join(selectorGroupingFixtureRoot, "swift", ".no-mistakes.yml"),
      framework: "swift",
      environment: "all",
      changedFiles: ["swift/Sources/App/Value.swift"],
    };
    const swiftDirect = await require("../index.js").testsPlan(swiftOptions);
    const swiftBatched = await require("../index.js").analyzeProject({
      root: swiftOptions.root,
      config: swiftOptions.config,
      reports: [
        {
          type: "testsPlan",
          framework: swiftOptions.framework,
          environment: swiftOptions.environment,
          changedFiles: swiftOptions.changedFiles,
        },
      ],
    });
    assert.deepEqual(swiftBatched.reports[0].result, swiftDirect);
    assert.deepEqual(
      swiftDirect.executionTargets.map(({ runnerArgs, testFiles }) => ({ runnerArgs, testFiles })),
      [
        {
          runnerArgs: ["--package-path", "swift", "--filter", "AlphaTests"],
          testFiles: ["swift/Tests/AlphaTests/Alpha.swift"],
        },
        {
          runnerArgs: ["--package-path", "swift", "--filter", "BetaTests"],
          testFiles: ["swift/Tests/BetaTests/Beta.swift"],
        },
      ],
    );
  },
);

test(
  "compiled internal N-API lock serializes separate Node processes",
  { skip: !compiledAddonPath },
  async () => {
    const directory = await mkdtemp(join(tmpdir(), "no-mistakes-napi-lock-"));
    const lockPath = join(directory, "artifact.lock");
    const native = require(addonPath);
    const child = fork(lockHolderFixture, [lockPath], {
      env: { ...process.env, NO_MISTAKES_TEST_NAPI_ADDON_PATH: addonPath },
      stdio: ["ignore", "ignore", "inherit", "ipc"],
    });
    let parentToken;
    try {
      const [message] = await once(child, "message");
      assert.equal(message, "locked");
      await assert.rejects(
        native.acquirePlanningArtifactLock(lockPath),
        /planning artifact lock is busy/,
      );

      const exited = once(child, "exit");
      child.send("release");
      assert.deepEqual(await exited, [0, null]);
      parentToken = await native.acquirePlanningArtifactLock(lockPath);
      await native.releasePlanningArtifactLock(parentToken);
      parentToken = undefined;
    } finally {
      if (parentToken !== undefined) await native.releasePlanningArtifactLock(parentToken);
      child.kill();
      await rm(directory, { recursive: true, force: true });
    }
  },
);

test(
  "compiled internal N-API lock is released when a worker terminates",
  { skip: !compiledAddonPath },
  async () => {
    const directory = await mkdtemp(join(tmpdir(), "no-mistakes-napi-worker-lock-"));
    const lockPath = join(directory, "artifact.lock");
    const worker = new Worker(workerLockHolderFixture, {
      workerData: { addonPath, lockPath },
    });
    let child;
    try {
      const [message] = await once(worker, "message");
      assert.equal(message, "locked");
      assert.equal(await worker.terminate(), 1);

      child = fork(lockHolderFixture, [lockPath], {
        env: { ...process.env, NO_MISTAKES_TEST_NAPI_ADDON_PATH: addonPath },
        stdio: ["ignore", "ignore", "inherit", "ipc"],
      });
      const [childMessage] = await Promise.race([
        once(child, "message"),
        delay(2_000, undefined, { ref: false }).then(() => {
          throw new Error("terminated worker retained its planning artifact lock");
        }),
      ]);
      assert.equal(childMessage, "locked");
      const exited = once(child, "exit");
      child.send("release");
      assert.deepEqual(await exited, [0, null]);
      child = undefined;
    } finally {
      await worker.terminate();
      child?.kill();
      await rm(directory, { recursive: true, force: true });
    }
  },
);

test(
  "compiled internal N-API rename preserves an existing destination and claims a vacant one",
  { skip: !compiledAddonPath },
  async () => {
    const directory = await mkdtemp(join(tmpdir(), "no-mistakes-napi-rename-"));
    const source = join(directory, "source");
    const destination = join(directory, "destination");
    try {
      await mkdir(source);
      await mkdir(destination);
      const native = require(addonPath);
      assert.equal(await native.renameNoReplace(source, destination), false);
      assert.equal((await stat(source)).isDirectory(), true);
      assert.equal((await stat(destination)).isDirectory(), true);
      await rm(destination, { recursive: true });
      assert.equal(await native.renameNoReplace(source, destination), true);
      await assert.rejects(stat(source), { code: "ENOENT" });
      assert.equal((await stat(destination)).isDirectory(), true);
    } finally {
      await rm(directory, { recursive: true, force: true });
    }
  },
);

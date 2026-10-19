const { assert } = require("./context");

async function assertProxyQueries(api, collectionGap) {
  assert.deepEqual((await api.check({ jobs: 2 })).jobsPoolWarning, {
    code: "rayon-pool-size",
    message: "existing pool",
    requestedJobs: 2,
    effectiveJobs: 4,
  });
  assert.equal(Object.hasOwn(await api.check(), "jobsPoolWarning"), false);
  assert.deepEqual(
    await api.dependencies({
      files: ["a.mts"],
      timeout: 12,
      invocationTimeoutMs: 3500,
      lockTimeout: null,
      failOnLock: true,
    }),
    {
      command: "dependencies",
      options: {
        files: ["a.mts"],
        timeout: 12,
        invocationTimeoutMs: 3500,
        lockTimeout: null,
        failOnLock: true,
      },
    },
  );
  await assert.rejects(
    api.dependencies({ files: ["a.mts"], root: "__locked__", failOnLock: true }),
    /another no-mistakes invocation holds the lock/,
  );
  assert.equal((await api.dependents({ files: ["b.mts"] })).command, "dependents");
  assert.equal((await api.related({ files: ["c.mts"] })).command, "related");
  assert.deepEqual(
    await api.analyzeProject({
      timeout: 0,
      lockTimeout: 5,
      reports: [{ type: "dependencies", files: ["a.mts"] }],
    }),
    {
      command: "analyzeProject",
      options: {
        timeout: 0,
        lockTimeout: 5,
        reports: [{ type: "dependencies", files: ["a.mts"] }],
      },
    },
  );
  assert.deepEqual(
    await api.analyzeProject({
      reports: [{ type: "check", includeSuppressed: true }],
    }),
    {
      command: "analyzeProject",
      options: {
        reports: [{ type: "check", includeSuppressed: true }],
      },
    },
  );
  assert.equal((await api.symbols({ files: ["d.mts"], include: "both" })).options.include, "both");
  assert.equal(
    (
      await api.symbols({
        files: ["d.mts"],
        mode: "signature-impact",
        symbol: "handler",
      })
    ).options.mode,
    "signature-impact",
  );
  assert.deepEqual(await api.importUsages({ filters: ["src/**"] }), {
    command: "importUsages",
    options: { filters: ["src/**"] },
  });
  assert.deepEqual(await api.importers({ file: "a.ts", tests: true }), {
    command: "importers",
    options: { file: "a.ts", tests: true },
  });
  assert.equal((await api.exportsOf({ file: "a.ts" })).command, "exportsOf");
  assert.equal((await api.deadExports({ file: "a.ts", names: ["foo"] })).options.names[0], "foo");
  assert.equal(
    (await api.callSites({ file: "a.ts", exportName: "foo" })).options.exportName,
    "foo",
  );
  for (const name of ["importers", "exportsOf", "deadExports", "callSites"]) {
    const report = await api[name]({ file: "__partial__", exportName: "foo" });
    assert.equal(report.partial, true);
    assert.deepEqual(report.incompleteFiles, collectionGap.incompleteFiles);
  }
  assert.equal((await api.resolveCheck({ file: "a.ts" })).command, "resolveCheck");
  assert.deepEqual((await api.resolveCheck({ files: ["a.ts", "b.ts"] })).options.files, [
    "a.ts",
    "b.ts",
  ]);
  await assert.rejects(api.resolveCheck({ files: [] }), /files must contain at least one path/);
  await assert.rejects(
    api.resolveCheck({ file: "a.ts", files: ["b.ts"] }),
    /exactly one of file or files is required/,
  );
}

module.exports = { assertProxyQueries };

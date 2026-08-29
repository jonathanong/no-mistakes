const assert = require("node:assert/strict");
const test = globalThis.test || require("node:test").test;
const { chmod, mkdtemp, readFile, rm, stat, writeFile } = require("node:fs/promises");
const { tmpdir } = require("node:os");
const { join } = require("node:path");

const packageRoot = join(__dirname, "..");
const addonPath = join(packageRoot, "bin", "no-mistakes.node");
const indexPath = join(packageRoot, "index.js");
const { writePlanningImpactArtifacts } = require("../planning-impact-artifacts");

const aggregateResult = {
  reports: [
    { id: "dependencies", type: "dependencies", result: { files: ["a.mts"] } },
    { id: "dependents", type: "dependents", result: { files: [] } },
    { id: "symbols", type: "symbols", result: { files: [{ path: "a.mts" }] } },
    {
      id: "plan",
      type: "testsPlan",
      result: { changedFiles: ["a.mts"], selectedTests: [{ testFile: "a.test.mts" }] },
    },
  ],
};

async function privateDirectory(prefix) {
  const directory = await mkdtemp(join(tmpdir(), prefix));
  await chmod(directory, 0o700);
  return directory;
}

test("writes the compatible four-report contract from one prepared analysis", async () => {
  const directory = await privateDirectory("no-mistakes-impact-");
  const manifest = join(directory, "changed-files.txt");
  const calls = [];
  try {
    await writeFile(manifest, "backend/a.mts\nREADME.md\nbackend/a.mts\n");
    const result = await writePlanningImpactArtifacts(
      { root: "/repo", changedFilesManifest: manifest, outputDirectory: directory },
      async (options) => {
        calls.push(options);
        return aggregateResult;
      },
    );

    assert.deepEqual(calls, [
      {
        root: "/repo",
        reports: [
          {
            id: "dependencies",
            type: "dependencies",
            files: ["backend/a.mts"],
            depth: 1,
            relationships: ["import", "workspace"],
          },
          {
            id: "dependents",
            type: "dependents",
            files: ["backend/a.mts"],
            depth: 1,
            relationships: ["import", "workspace"],
          },
          { id: "symbols", type: "symbols", files: ["backend/a.mts"], include: "both" },
          {
            id: "plan",
            type: "testsPlan",
            framework: "vitest",
            environment: "prePush",
            changedFiles: ["backend/a.mts", "README.md"],
          },
        ],
      },
    ]);
    assert.deepEqual(result.plan, aggregateResult.reports[3].result);
    assert.deepEqual(JSON.parse(await readFile(join(directory, "plan.json"), "utf8")), {
      changed_files: ["a.mts"],
      selected_tests: [{ test_file: "a.test.mts" }],
    });
    for (const name of ["dependencies", "dependents", "symbols", "plan"]) {
      assert.equal(await readFile(join(directory, `${name}.status`), "utf8"), "0\n");
      assert.equal(await readFile(join(directory, `${name}.stderr`), "utf8"), "");
      assert.equal((await stat(join(directory, `${name}.json`))).mode & 0o777, 0o600);
      assert.equal((await stat(join(directory, `${name}.stderr`))).mode & 0o777, 0o600);
      assert.equal((await stat(join(directory, `${name}.status`))).mode & 0o777, 0o600);
    }
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

test("writes empty structural artifacts for documentation-only changes", async () => {
  const directory = await privateDirectory("no-mistakes-impact-");
  const manifest = join(directory, "changed-files.txt");
  try {
    await writeFile(manifest, "README.md\n");
    const result = await writePlanningImpactArtifacts(
      { root: "/repo", changedFilesManifest: manifest, outputDirectory: directory, broad: true },
      async (options) => {
        assert.deepEqual(options.reports, [
          {
            id: "plan",
            type: "testsPlan",
            framework: "vitest",
            environment: "prePush",
            changedFiles: ["README.md"],
          },
        ]);
        return { reports: [aggregateResult.reports[3]] };
      },
    );
    assert.deepEqual(result.dependencies, {
      roots: [],
      files: [],
      diagnostics: [],
      tsconfig_provenance: [],
    });
    assert.deepEqual(result.symbols, { roots: [], files: [] });
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

test("rejects unsafe paths and records bounded failures without stale JSON", async () => {
  const directory = await privateDirectory("no-mistakes-impact-");
  const manifest = join(directory, "changed-files.txt");
  try {
    await writeFile(manifest, "../outside.mts\n");
    await writeFile(join(directory, "dependencies.json"), "stale");
    await assert.rejects(
      writePlanningImpactArtifacts(
        { root: "/repo", changedFilesManifest: manifest, outputDirectory: directory },
        async () => aggregateResult,
      ),
      /repository-relative/,
    );
    for (const name of ["dependencies", "dependents", "symbols", "plan"]) {
      await assert.rejects(readFile(join(directory, `${name}.json`), "utf8"), /ENOENT/);
      assert.equal(await readFile(join(directory, `${name}.status`), "utf8"), "1\n");
      assert.ok(
        Buffer.byteLength(await readFile(join(directory, `${name}.stderr`), "utf8")) <= 4096,
      );
    }

    await writeFile(manifest, "a.mts\n");
    await assert.rejects(
      writePlanningImpactArtifacts(
        { root: "/repo", changedFilesManifest: manifest, outputDirectory: directory },
        async () => {
          throw new Error(`failure${"💥".repeat(3000)}`);
        },
      ),
    );
    const diagnostic = await readFile(join(directory, "plan.stderr"), "utf8");
    assert.ok(Buffer.byteLength(diagnostic) <= 4096);
    assert.ok(!diagnostic.endsWith("�"));
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

test("requires a 0700 output directory and a manifest contained by it", async () => {
  const directory = await privateDirectory("no-mistakes-impact-");
  const outside = await privateDirectory("no-mistakes-impact-outside-");
  try {
    await writeFile(join(outside, "changed-files.txt"), "a.mts\n");
    await assert.rejects(
      writePlanningImpactArtifacts(
        {
          root: "/repo",
          changedFilesManifest: join(outside, "changed-files.txt"),
          outputDirectory: directory,
        },
        async () => aggregateResult,
      ),
      /inside the private output directory/,
    );
    await chmod(directory, 0o750);
    await assert.rejects(
      writePlanningImpactArtifacts(
        {
          root: "/repo",
          changedFilesManifest: join(directory, "changed-files.txt"),
          outputDirectory: directory,
        },
        async () => aggregateResult,
      ),
      /mode 0700/,
    );
  } finally {
    await rm(directory, { recursive: true, force: true });
    await rm(outside, { recursive: true, force: true });
  }
});

test("exports the artifact writer through the public async Node API", async () => {
  const directory = await privateDirectory("no-mistakes-impact-");
  const manifest = join(directory, "changed-files.txt");
  const previous = require.extensions[".node"];
  delete require.cache[require.resolve(indexPath)];
  delete require.cache[require.resolve(addonPath)];
  require.extensions[".node"] = (module, filename) => {
    assert.equal(filename, addonPath);
    module.exports = {
      analyzeProjectJson: async () => JSON.stringify(aggregateResult),
    };
  };
  try {
    await writeFile(manifest, "a.mts\n");
    const api = require(indexPath);
    const result = await api.writePlanningImpactArtifacts({
      root: "/repo",
      changedFilesManifest: manifest,
      outputDirectory: directory,
    });
    assert.deepEqual(result.dependencies, { files: ["a.mts"] });
  } finally {
    delete require.cache[require.resolve(indexPath)];
    delete require.cache[require.resolve(addonPath)];
    if (previous) require.extensions[".node"] = previous;
    else delete require.extensions[".node"];
    await rm(directory, { recursive: true, force: true });
  }
});

const assert = require("node:assert/strict");
const { execFileSync } = require("node:child_process");
const { constants } = require("node:fs");
const test = globalThis.test || require("node:test").test;
const {
  chmod,
  link,
  lstat,
  mkdir,
  mkdtemp,
  readFile,
  readdir,
  rename,
  rm,
  stat,
  symlink,
  unlink,
  writeFile,
} = require("node:fs/promises");
const { tmpdir } = require("node:os");
const { basename, dirname, join } = require("node:path");
const { pathToFileURL } = require("node:url");

const packageRoot = join(__dirname, "..");
const repositoryRoot = join(packageRoot, "..", "..");
const addonPath = join(packageRoot, "bin", "no-mistakes.node");
const indexPath = join(packageRoot, "index.js");
const esmIndexPath = join(packageRoot, "index.mjs");
const {
  writePlanningImpactArtifacts: writePlanningImpactArtifactsInternal,
} = require("../planning-impact-artifacts");
const { preserveFailureReportingError } = require("../planning-impact-artifacts-errors");

// The published API supplies the native no-replace rename primitive. These direct helper tests
// use Node's rename except where a deterministic race supplies its own fail-closed primitive.
const writePlanningImpactArtifacts = (options, analyzeProject) =>
  writePlanningImpactArtifactsInternal(options, analyzeProject, async (from, to) => {
    await rename(from, to);
    return true;
  });

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

async function withFsOverride(overrides, callback, renameNoReplace) {
  const fs = require("node:fs/promises");
  const originals = Object.fromEntries(Object.keys(overrides).map((name) => [name, fs[name]]));
  const restoreWithoutReplacement =
    renameNoReplace ||
    (async (from, to) => {
      await fs.rename(from, to);
      return true;
    });
  Object.assign(fs, overrides);
  const helperModules = [
    "../planning-impact-artifacts",
    "../planning-impact-artifacts-files",
    "../planning-impact-artifacts-inputs",
  ];
  for (const helperModule of helperModules) delete require.cache[require.resolve(helperModule)];
  try {
    const artifacts = require("../planning-impact-artifacts");
    return await callback({
      ...artifacts,
      writePlanningImpactArtifacts: async (options, analyzeProject) =>
        artifacts.writePlanningImpactArtifacts(options, analyzeProject, restoreWithoutReplacement),
    });
  } finally {
    Object.assign(fs, originals);
    for (const helperModule of helperModules) delete require.cache[require.resolve(helperModule)];
  }
}

test("writes the compatible four-report contract from one prepared analysis", async () => {
  const directory = await privateDirectory("no-mistakes-impact-");
  const manifest = join(directory, "changed-files.txt");
  const calls = [];
  try {
    await writeFile(manifest, "backend/a.mts\nREADME.md\nbackend/a.mts\n");
    const result = await writePlanningImpactArtifacts(
      {
        root: "/repo",
        changedFilesManifest: manifest,
        outputDirectory: directory,
        timeout: 3,
        lockTimeout: null,
        failOnLock: true,
        jobs: 2,
        profile: "ci",
      },
      async (options) => {
        calls.push(options);
        return aggregateResult;
      },
    );

    assert.deepEqual(calls, [
      {
        root: "/repo",
        timeout: 3,
        lockTimeout: null,
        failOnLock: true,
        jobs: 2,
        profile: "ci",
        reports: [
          {
            id: "dependencies",
            type: "dependencies",
            files: [{ file: "backend/a.mts" }],
            depth: 1,
            relationships: ["import", "workspace"],
          },
          {
            id: "dependents",
            type: "dependents",
            files: [{ file: "backend/a.mts" }],
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

test("preserves literal manifest paths and traverses hash filenames as files", async () => {
  const directory = await privateDirectory("no-mistakes-impact-");
  const manifest = join(directory, "changed-files.txt");
  const calls = [];
  try {
    await writeFile(manifest, " src.mts \nfoo.ts#literal\n#bar.ts\n\n");
    await writePlanningImpactArtifacts(
      { root: "/repo", changedFilesManifest: manifest, outputDirectory: directory },
      async (options) => {
        calls.push(options);
        return aggregateResult;
      },
    );
    assert.deepEqual(calls[0].reports, [
      {
        id: "dependencies",
        type: "dependencies",
        files: [{ file: "foo.ts#literal" }, { file: "#bar.ts" }],
        depth: 1,
        relationships: ["import", "workspace"],
      },
      {
        id: "dependents",
        type: "dependents",
        files: [{ file: "foo.ts#literal" }, { file: "#bar.ts" }],
        depth: 1,
        relationships: ["import", "workspace"],
      },
      { id: "symbols", type: "symbols", files: ["foo.ts#literal", "#bar.ts"], include: "both" },
      {
        id: "plan",
        type: "testsPlan",
        framework: "vitest",
        environment: "prePush",
        changedFiles: [" src.mts ", "foo.ts#literal", "#bar.ts"],
      },
    ]);
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

test("uses POSIX modes only on platforms where Node reports them reliably", () => {
  const {
    isPrivateDirectory,
    isPrivateRegularFile,
    privatePermissionError,
  } = require("../planning-impact-artifacts-privacy");
  const directory = { isDirectory: () => true, mode: 0o40700 };
  const file = { isFile: () => true, mode: 0o100600 };

  assert.equal(isPrivateDirectory(directory, "linux"), true);
  assert.equal(isPrivateRegularFile(file, "linux"), true);
  assert.equal(isPrivateDirectory(directory, "win32"), false);
  assert.equal(isPrivateRegularFile(file, "win32"), false);
  assert.throws(() => {
    throw privatePermissionError("output directory", 0o700, "win32");
  }, /unavailable on Windows/);
});

test("publishes 0600 artifacts under a restrictive umask and restores it", async () => {
  const directory = await privateDirectory("no-mistakes-impact-");
  const manifest = join(directory, "changed-files.txt");
  const originalUmask = process.umask();
  try {
    await writeFile(manifest, "a.mts\n");
    process.umask(0o600);
    await writePlanningImpactArtifacts(
      { root: "/repo", changedFilesManifest: manifest, outputDirectory: directory },
      async () => aggregateResult,
    );
    assert.equal((await stat(join(directory, "plan.json"))).mode & 0o777, 0o600);
  } finally {
    process.umask(originalUmask);
    await rm(directory, { recursive: true, force: true });
  }
  assert.equal(process.umask(), originalUmask);
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

test("keeps deleted structural files out of symbols while following file symlinks", async () => {
  const root = await privateDirectory("no-mistakes-impact-root-");
  const directory = await privateDirectory("no-mistakes-impact-");
  const manifest = join(directory, "changed-files.txt");
  let request;
  try {
    await writeFile(join(root, "present.mts"), "export const present = true;\n");
    await symlink("present.mts", join(root, "linked.mts"));
    await mkdir(join(root, "folder.mts"));
    await writeFile(manifest, "present.mts\nlinked.mts\ndeleted.mts\nfolder.mts\nREADME.md\n");
    await writePlanningImpactArtifacts(
      { root, changedFilesManifest: manifest, outputDirectory: directory },
      async (options) => {
        request = options;
        return aggregateResult;
      },
    );
    assert.deepEqual(request.reports[0].files, [
      { file: "present.mts" },
      { file: "linked.mts" },
      { file: "deleted.mts" },
      { file: "folder.mts" },
    ]);
    assert.deepEqual(request.reports[2].files, ["present.mts", "linked.mts"]);
    assert.deepEqual(request.reports[3].changedFiles, [
      "present.mts",
      "linked.mts",
      "deleted.mts",
      "folder.mts",
      "README.md",
    ]);
  } finally {
    await rm(root, { recursive: true, force: true });
    await rm(directory, { recursive: true, force: true });
  }
});

test("omits the symbols request when every structural file was deleted", async () => {
  const root = await privateDirectory("no-mistakes-impact-root-");
  const directory = await privateDirectory("no-mistakes-impact-");
  const manifest = join(directory, "changed-files.txt");
  try {
    await writeFile(manifest, "deleted.mts\n");
    const result = await writePlanningImpactArtifacts(
      { root, changedFilesManifest: manifest, outputDirectory: directory },
      async (options) => {
        assert.deepEqual(
          options.reports.map((report) => report.id),
          ["dependencies", "dependents", "plan"],
        );
        return {
          reports: [
            aggregateResult.reports[0],
            aggregateResult.reports[1],
            aggregateResult.reports[3],
          ],
        };
      },
    );
    assert.deepEqual(result.symbols, { roots: [], files: [] });
  } finally {
    await rm(root, { recursive: true, force: true });
    await rm(directory, { recursive: true, force: true });
  }
});

test("keeps source symlinks in the symbols report", async () => {
  const root = join(
    packageRoot,
    "..",
    "..",
    "fixtures",
    "codebase",
    "dependencies",
    "graph-files-dual-alias",
    "fixture",
  );
  const directory = await privateDirectory("no-mistakes-impact-");
  const manifest = join(directory, "changed-files.txt");
  let request;
  try {
    await writeFile(manifest, "a.ts\n");
    await writePlanningImpactArtifacts(
      { root, changedFilesManifest: manifest, outputDirectory: directory },
      async (options) => {
        request = options;
        return aggregateResult;
      },
    );
    assert.deepEqual(request.reports[2].files, ["a.ts"]);
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

test("preserves the analysis error when checking the root or changed-file identity fails", async () => {
  const directory = await privateDirectory("no-mistakes-impact-");
  const manifest = join(directory, "changed-files.txt");
  const root = await privateDirectory("no-mistakes-impact-root-");
  const fs = require("node:fs/promises");
  const originalStat = fs.stat;
  try {
    await writeFile(join(root, "error.mts"), "export const error = true;\n");
    await writeFile(manifest, "error.mts\n");
    const injected = new Error("root identity unavailable");
    injected.code = "EACCES";
    await withFsOverride(
      {
        stat: async (path, ...args) => {
          if (path === root) throw injected;
          return originalStat(path, ...args);
        },
      },
      async ({ writePlanningImpactArtifacts: writeArtifacts }) => {
        await assert.rejects(
          writeArtifacts(
            { root, changedFilesManifest: manifest, outputDirectory: directory },
            async () => aggregateResult,
          ),
          (error) => error === injected,
        );
      },
    );

    const changedFile = join(root, "error.mts");
    const fileError = new Error("changed-file identity unavailable");
    fileError.code = "EIO";
    await withFsOverride(
      {
        stat: async (path, ...args) => {
          if (path === changedFile) throw fileError;
          return originalStat(path, ...args);
        },
      },
      async ({ writePlanningImpactArtifacts: writeArtifacts }) => {
        await assert.rejects(
          writeArtifacts(
            { root, changedFilesManifest: manifest, outputDirectory: directory },
            async () => aggregateResult,
          ),
          (error) => error === fileError,
        );
      },
    );
  } finally {
    await rm(directory, { recursive: true, force: true });
    await rm(root, { recursive: true, force: true });
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

    for (const unsafePath of [
      "/outside.mts",
      "C:\\outside.mts",
      "\\\\server\\share\\outside.mts",
    ]) {
      await writeFile(manifest, `${unsafePath}\n`);
      await assert.rejects(
        writePlanningImpactArtifacts(
          { root: "/repo", changedFilesManifest: manifest, outputDirectory: directory },
          async () => aggregateResult,
        ),
        /repository-relative/,
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

test("replaces stale success artifacts when manifest canonicalization fails", async () => {
  const directory = await privateDirectory("no-mistakes-impact-");
  const manifest = join(directory, "changed-files.txt");
  try {
    await writeFile(manifest, "a.mts\n");
    await writePlanningImpactArtifacts(
      { root: "/repo", changedFilesManifest: manifest, outputDirectory: directory },
      async () => aggregateResult,
    );
    await unlink(manifest);

    await assert.rejects(
      writePlanningImpactArtifacts(
        { root: "/repo", changedFilesManifest: manifest, outputDirectory: directory },
        async () => aggregateResult,
      ),
      /ENOENT/u,
    );
    for (const name of ["dependencies", "dependents", "symbols", "plan"]) {
      await assert.rejects(readFile(join(directory, `${name}.json`), "utf8"), /ENOENT/u);
      assert.equal(await readFile(join(directory, `${name}.status`), "utf8"), "1\n");
      assert.match(await readFile(join(directory, `${name}.stderr`), "utf8"), /ENOENT/u);
    }
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

test("serializes invalidation, analysis, and publication for one output directory", async () => {
  const directory = await privateDirectory("no-mistakes-impact-");
  const manifest = join(directory, "changed-files.txt");
  let beginFirst;
  let releaseFirst;
  const firstStarted = new Promise((resolveStarted) => {
    beginFirst = resolveStarted;
  });
  const firstMayFinish = new Promise((resolveFinish) => {
    releaseFirst = resolveFinish;
  });
  let secondStarted = false;
  try {
    await writeFile(manifest, "a.mts\n");
    const first = writePlanningImpactArtifacts(
      { root: "/repo", changedFilesManifest: manifest, outputDirectory: directory },
      async () => {
        beginFirst();
        await firstMayFinish;
        return aggregateResult;
      },
    );
    await firstStarted;
    const second = writePlanningImpactArtifacts(
      { root: "/repo", changedFilesManifest: manifest, outputDirectory: directory },
      async () => {
        secondStarted = true;
        return aggregateResult;
      },
    );
    await new Promise((resolveTurn) => setImmediate(resolveTurn));
    assert.equal(secondStarted, false);

    releaseFirst();
    await first;
    await second;
    assert.equal(secondStarted, true);
  } finally {
    releaseFirst();
    await rm(directory, { recursive: true, force: true });
  }
});

test("queues a concurrent call before resolving a temporarily parked output path", async () => {
  const directory = await privateDirectory("no-mistakes-impact-queue-");
  const manifest = join(directory, "changed-files.txt");
  const fs = require("node:fs/promises");
  const originalRename = fs.rename;
  const canonicalDirectory = await fs.realpath(directory);
  let announceParking;
  let resumeParking;
  const parkingStarted = new Promise((resolveStarted) => {
    announceParking = resolveStarted;
  });
  const parkingMayFinish = new Promise((resolveFinish) => {
    resumeParking = resolveFinish;
  });
  let parkedOnce = false;
  try {
    await writeFile(manifest, "a.mts\n");
    await withFsOverride(
      {
        rename: async (from, to) => {
          const result = await originalRename(from, to);
          if (!parkedOnce && from === canonicalDirectory) {
            parkedOnce = true;
            announceParking();
            await parkingMayFinish;
          }
          return result;
        },
      },
      async ({ writePlanningImpactArtifacts: writeArtifacts }) => {
        const first = writeArtifacts(
          { root: "/repo", changedFilesManifest: manifest, outputDirectory: directory },
          async () => aggregateResult,
        );
        await parkingStarted;
        let secondSettled = false;
        const second = writeArtifacts(
          { root: "/repo", changedFilesManifest: manifest, outputDirectory: directory },
          async () => aggregateResult,
        ).finally(() => {
          secondSettled = true;
        });
        await new Promise((resolveTurn) => setImmediate(resolveTurn));
        assert.equal(secondSettled, false);
        resumeParking();
        await Promise.all([first, second]);
      },
    );
  } finally {
    resumeParking();
    await rm(directory, { recursive: true, force: true });
  }
});

test("serializes canonical and symlink aliases while the output is parked", async () => {
  const directory = await privateDirectory("no-mistakes-impact-alias-queue-");
  const alias = `${directory}-alias`;
  const manifest = join(directory, "changed-files.txt");
  const fs = require("node:fs/promises");
  const originalRename = fs.rename;
  const canonicalDirectory = await fs.realpath(directory);
  let announceParking;
  let resumeParking;
  const parkingStarted = new Promise((resolveStarted) => {
    announceParking = resolveStarted;
  });
  const parkingMayFinish = new Promise((resolveFinish) => {
    resumeParking = resolveFinish;
  });
  let parkedOnce = false;
  try {
    await symlink(canonicalDirectory, alias);
    await writeFile(manifest, "a.mts\n");
    await withFsOverride(
      {
        rename: async (from, to) => {
          const result = await originalRename(from, to);
          if (!parkedOnce && from === canonicalDirectory) {
            parkedOnce = true;
            announceParking();
            await parkingMayFinish;
          }
          return result;
        },
      },
      async ({ writePlanningImpactArtifacts: writeArtifacts }) => {
        const first = writeArtifacts(
          { root: "/repo", changedFilesManifest: manifest, outputDirectory: directory },
          async () => aggregateResult,
        );
        await parkingStarted;
        let aliasSettled = false;
        const second = writeArtifacts(
          { root: "/repo", changedFilesManifest: manifest, outputDirectory: alias },
          async () => aggregateResult,
        ).finally(() => {
          aliasSettled = true;
        });
        await new Promise((resolveTurn) => setImmediate(resolveTurn));
        assert.equal(aliasSettled, false);
        resumeParking();
        await Promise.all([first, second]);
      },
    );
  } finally {
    resumeParking();
    await rm(alias, { force: true });
    await rm(directory, { recursive: true, force: true });
  }
});

test("strips a UTF-8 BOM from the changed-files manifest", async () => {
  const directory = await privateDirectory("no-mistakes-impact-bom-");
  const manifest = join(directory, "changed-files.txt");
  const fixture = JSON.parse(
    await readFile(
      join(
        repositoryRoot,
        "fixtures",
        "napi",
        "planning-impact-artifacts",
        "utf8-bom-manifest.json",
      ),
      "utf8",
    ),
  );
  const calls = [];
  try {
    await writeFile(manifest, Buffer.from(fixture.hex, "hex"));
    await writePlanningImpactArtifacts(
      { root: "/repo", changedFilesManifest: manifest, outputDirectory: directory },
      async (options) => {
        calls.push(options);
        return aggregateResult;
      },
    );
    assert.deepEqual(calls[0].reports[3].changedFiles, ["a.mts"]);
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

test("rejects an invalid UTF-8 changed-files manifest with failure artifacts", async () => {
  const directory = await privateDirectory("no-mistakes-impact-utf8-");
  const manifest = join(directory, "changed-files.txt");
  const fixture = JSON.parse(
    await readFile(
      join(
        repositoryRoot,
        "fixtures",
        "napi",
        "planning-impact-artifacts",
        "invalid-utf8-manifest.json",
      ),
      "utf8",
    ),
  );
  let analyzed = false;
  try {
    await writeFile(manifest, Buffer.from(fixture.hex, "hex"));
    await assert.rejects(
      writePlanningImpactArtifacts(
        { root: "/repo", changedFilesManifest: manifest, outputDirectory: directory },
        async () => {
          analyzed = true;
          return aggregateResult;
        },
      ),
      /manifest must be valid UTF-8/,
    );
    assert.equal(analyzed, false);
    assert.equal(await readFile(join(directory, "dependencies.status"), "utf8"), "1\n");
    assert.match(
      await readFile(join(directory, "dependencies.stderr"), "utf8"),
      /manifest must be valid UTF-8/,
    );
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

test("replaces symlink and hardlink artifact destinations without touching their victims", async () => {
  const directory = await privateDirectory("no-mistakes-impact-");
  const manifest = join(directory, "changed-files.txt");
  const victim = join(directory, "victim.txt");
  try {
    await writeFile(manifest, "a.mts\n");
    await writeFile(victim, "protected");
    await symlink(victim, join(directory, "dependencies.json"));
    await link(victim, join(directory, "plan.json"));
    await writePlanningImpactArtifacts(
      { root: "/repo", changedFilesManifest: manifest, outputDirectory: directory },
      async () => aggregateResult,
    );
    assert.equal(await readFile(victim, "utf8"), "protected");
    assert.notEqual((await stat(join(directory, "plan.json"))).ino, (await stat(victim)).ino);
    assert.deepEqual(JSON.parse(await readFile(join(directory, "dependencies.json"), "utf8")), {
      files: ["a.mts"],
    });
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

test("removes unsafe failure destinations without touching their victims", async () => {
  const directory = await privateDirectory("no-mistakes-impact-");
  const manifest = join(directory, "changed-files.txt");
  const victim = join(directory, "victim.txt");
  try {
    await writeFile(manifest, "../outside.mts\n");
    await writeFile(victim, "protected");
    await symlink(victim, join(directory, "dependencies.stderr"));
    await link(victim, join(directory, "plan.status"));
    await assert.rejects(
      writePlanningImpactArtifacts(
        { root: "/repo", changedFilesManifest: manifest, outputDirectory: directory },
        async () => aggregateResult,
      ),
      /repository-relative/,
    );
    assert.equal(await readFile(victim, "utf8"), "protected");
    assert.equal(
      await readFile(join(directory, "dependencies.stderr"), "utf8"),
      "Error: changed file must be repository-relative: ../outside.mts\n",
    );
    assert.equal(await readFile(join(directory, "plan.status"), "utf8"), "1\n");
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

test("reads the canonical manifest after a symlink is retargeted", async () => {
  const directory = await privateDirectory("no-mistakes-impact-");
  const manifest = join(directory, "changed-files.txt");
  const first = join(directory, "first.txt");
  const second = join(directory, "second.txt");
  const fs = require("node:fs/promises");
  const realpath = fs.realpath;
  try {
    await writeFile(first, "first.mts\n");
    await writeFile(second, "second.mts\n");
    await symlink(first, manifest);
    await withFsOverride(
      {
        realpath: async (path) => {
          const resolved = await realpath(path);
          if (path === manifest) {
            await unlink(manifest);
            await symlink(second, manifest);
          }
          return resolved;
        },
      },
      async ({ writePlanningImpactArtifacts: writeArtifacts }) => {
        await writeArtifacts(
          { root: "/repo", changedFilesManifest: manifest, outputDirectory: directory },
          async (options) => {
            assert.deepEqual(options.reports[0].files, [{ file: "first.mts" }]);
            return aggregateResult;
          },
        );
      },
    );
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

test("rejects a manifest symlink whose canonical target escapes the output directory", async () => {
  const directory = await privateDirectory("no-mistakes-impact-");
  const outside = await privateDirectory("no-mistakes-impact-outside-");
  const manifest = join(directory, "changed-files.txt");
  const outsideManifest = join(outside, "changed-files.txt");
  try {
    await writeFile(outsideManifest, "a.mts\n");
    await symlink(outsideManifest, manifest);
    await assert.rejects(
      writePlanningImpactArtifacts(
        { root: "/repo", changedFilesManifest: manifest, outputDirectory: directory },
        async () => aggregateResult,
      ),
      /inside the private output directory/,
    );
    for (const name of ["dependencies", "dependents", "symbols", "plan"]) {
      assert.equal(await readFile(join(directory, `${name}.status`), "utf8"), "1\n");
    }
  } finally {
    await rm(directory, { recursive: true, force: true });
    await rm(outside, { recursive: true, force: true });
  }
});

test("does not enable failure writes through a reserved manifest symlink alias", async () => {
  const directory = await privateDirectory("no-mistakes-impact-");
  const manifest = join(directory, "changed-files.txt");
  const reserved = join(directory, "plan.json");
  let analyzed = false;
  try {
    await mkdir(reserved);
    await symlink(reserved, manifest);
    await assert.rejects(
      writePlanningImpactArtifacts(
        { root: "/repo", changedFilesManifest: manifest, outputDirectory: directory },
        async () => {
          analyzed = true;
          return aggregateResult;
        },
      ),
      /regular file/,
    );
    assert.equal(analyzed, false);
    for (const name of ["dependencies", "dependents", "symbols", "plan"]) {
      await assert.rejects(lstat(join(directory, `${name}.status`)), /ENOENT/);
    }
    assert.equal((await lstat(reserved)).isDirectory(), true);
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

test("closes the output descriptor when manifest normalization fails", async () => {
  const directory = await privateDirectory("no-mistakes-impact-");
  const fs = require("node:fs/promises");
  const originalOpen = fs.open;
  const canonicalDirectory = await fs.realpath(directory);
  let opened = 0;
  let closed = 0;
  try {
    const malformed = join(directory, "malformed.txt");
    await writeFile(malformed, "\n");
    await withFsOverride(
      {
        open: async (path, ...args) => {
          const handle = await originalOpen(path, ...args);
          if (path === canonicalDirectory) {
            opened += 1;
            const originalClose = handle.close.bind(handle);
            handle.close = async (...closeArgs) => {
              closed += 1;
              return originalClose(...closeArgs);
            };
          }
          return handle;
        },
      },
      async ({ writePlanningImpactArtifacts: writeArtifacts }) => {
        for (const changedFilesManifest of [undefined, null, 42, join(directory, "missing.txt")]) {
          await assert.rejects(
            writeArtifacts(
              { root: "/repo", changedFilesManifest, outputDirectory: directory },
              async () => aggregateResult,
            ),
          );
        }
        await assert.rejects(
          writeArtifacts(
            { root: "/repo", changedFilesManifest: malformed, outputDirectory: directory },
            async () => aggregateResult,
          ),
          /empty/,
        );
      },
    );
    assert.ok(opened > 0);
    assert.equal(closed, opened);
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

test("rejects a manifest hardlink to an outside file before analysis", async () => {
  const directory = await privateDirectory("no-mistakes-impact-");
  const outside = await privateDirectory("no-mistakes-impact-outside-");
  const manifest = join(directory, "changed-files.txt");
  const outsideManifest = join(outside, "changed-files.txt");
  let analyzed = false;
  try {
    await writeFile(outsideManifest, "a.mts\n");
    await link(outsideManifest, manifest);
    await assert.rejects(
      writePlanningImpactArtifacts(
        { root: "/repo", changedFilesManifest: manifest, outputDirectory: directory },
        async () => {
          analyzed = true;
          return aggregateResult;
        },
      ),
      /manifest changed during validation/,
    );
    assert.equal(analyzed, false);
    assert.equal(await readFile(outsideManifest, "utf8"), "a.mts\n");
    for (const name of ["dependencies", "dependents", "symbols", "plan"]) {
      assert.equal(await readFile(join(directory, `${name}.status`), "utf8"), "1\n");
    }
  } finally {
    await rm(directory, { recursive: true, force: true });
    await rm(outside, { recursive: true, force: true });
  }
});

test(
  "rejects a FIFO changed-files manifest without waiting for a writer",
  { skip: process.platform === "win32" },
  async () => {
    const directory = await privateDirectory("no-mistakes-impact-");
    const manifest = join(directory, "changed-files.txt");
    const {
      validateManifest,
      validateOutputDirectory,
    } = require("../planning-impact-artifacts-files");
    let output;
    try {
      execFileSync("mkfifo", [manifest]);
      output = await validateOutputDirectory(directory);
      await assert.rejects(
        validateManifest(output, manifest),
        /changed-files manifest must be a regular file/,
      );
    } finally {
      await output?.handle?.close().catch(() => {});
      await rm(directory, { recursive: true, force: true });
    }
  },
);

test(
  "opens special changed-files manifests nonblocking before rejecting them",
  { skip: constants.O_NONBLOCK === undefined },
  async () => {
    const directory = await privateDirectory("no-mistakes-impact-");
    const manifest = join(directory, "changed-files.txt");
    const fs = require("node:fs/promises");
    const originalOpen = fs.open;
    try {
      await writeFile(manifest, "a.mts\n");
      const canonicalManifest = await fs.realpath(manifest);
      let openedNonblocking = false;
      await withFsOverride(
        {
          open: async (path, flags, ...rest) => {
            if (path === canonicalManifest) {
              openedNonblocking = (flags & constants.O_NONBLOCK) !== 0;
              return {
                stat: async () => ({ isFile: () => false }),
                close: async () => {},
              };
            }
            return originalOpen(path, flags, ...rest);
          },
        },
        async ({ writePlanningImpactArtifacts: writeArtifacts }) => {
          await assert.rejects(
            writeArtifacts(
              { root: "/repo", changedFilesManifest: manifest, outputDirectory: directory },
              async () => aggregateResult,
            ),
            /changed-files manifest must be a regular file/,
          );
        },
      );
      assert.equal(openedNonblocking, true);
    } finally {
      await rm(directory, { recursive: true, force: true });
    }
  },
);

test("requires the private output directory to share its parent's filesystem", async () => {
  const directory = await privateDirectory("no-mistakes-impact-");
  const fs = require("node:fs/promises");
  const originalStat = fs.stat;
  try {
    const canonicalParent = dirname(await fs.realpath(directory));
    await withFsOverride(
      {
        stat: async (path, ...rest) => {
          const metadata = await originalStat(path, ...rest);
          if (path !== canonicalParent) return metadata;
          return { dev: metadata.dev + 1, isDirectory: () => true };
        },
      },
      async () => {
        const { validateOutputDirectory } = require("../planning-impact-artifacts-files");
        await assert.rejects(
          validateOutputDirectory(directory),
          /must share its parent's filesystem for atomic planning artifact publication/,
        );
      },
    );
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

test("rejects a manifest path swapped after it is opened", async () => {
  const directory = await privateDirectory("no-mistakes-impact-");
  const outside = await privateDirectory("no-mistakes-impact-outside-");
  const manifest = join(directory, "changed-files.txt");
  const outsideManifest = join(outside, "changed-files.txt");
  const fs = require("node:fs/promises");
  const originalOpen = fs.open;
  try {
    await writeFile(manifest, "a.mts\n");
    await writeFile(outsideManifest, "outside.mts\n");
    const canonicalManifest = await fs.realpath(manifest);
    let swapped = false;
    await withFsOverride(
      {
        open: async (path, ...args) => {
          const handle = await originalOpen(path, ...args);
          if (!swapped && path === canonicalManifest) {
            swapped = true;
            await unlink(manifest);
            await symlink(outsideManifest, manifest);
          }
          return handle;
        },
      },
      async ({ writePlanningImpactArtifacts: writeArtifacts }) => {
        await assert.rejects(
          writeArtifacts(
            { root: "/repo", changedFilesManifest: manifest, outputDirectory: directory },
            async () => aggregateResult,
          ),
          /manifest changed during validation/,
        );
      },
    );
  } finally {
    await rm(directory, { recursive: true, force: true });
    await rm(outside, { recursive: true, force: true });
  }
});

test("rejects an output directory replacement before publishing artifacts", async () => {
  const directory = await privateDirectory("no-mistakes-impact-");
  const victim = await privateDirectory("no-mistakes-impact-victim-");
  const manifest = join(directory, "changed-files.txt");
  const fs = require("node:fs/promises");
  const originalRealpath = fs.realpath;
  const originalRename = fs.rename;
  const canonicalDirectory = await originalRealpath(directory);
  try {
    await writeFile(manifest, "a.mts\n");
    await writeFile(join(victim, "dependencies.json"), "protected");
    let swapped = false;
    await withFsOverride(
      {
        rename: async (from, to) => {
          const result = await originalRename(from, to);
          if (!swapped && to.endsWith("dependencies.json")) {
            swapped = true;
            await symlink(victim, canonicalDirectory);
          }
          return result;
        },
      },
      async ({ writePlanningImpactArtifacts: writeArtifacts }) => {
        await assert.rejects(
          writeArtifacts(
            { root: "/repo", changedFilesManifest: manifest, outputDirectory: directory },
            async () => aggregateResult,
          ),
          (error) =>
            error instanceof AggregateError &&
            /recover the parked directory/u.test(error.message) &&
            /output directory (?:path|descriptor) changed/u.test(error.cause.message),
        );
      },
    );
    assert.equal(await readFile(join(victim, "dependencies.json"), "utf8"), "protected");
  } finally {
    await rm(directory, { recursive: true, force: true });
    for (const entry of await readdir(dirname(canonicalDirectory))) {
      if (entry.startsWith(`.${basename(canonicalDirectory)}.planning-impact-`)) {
        await rm(join(dirname(canonicalDirectory), entry), { recursive: true, force: true });
      }
    }
    await rm(victim, { recursive: true, force: true });
  }
});

test("rejects an output path whose canonical directory changes", async () => {
  const directory = await privateDirectory("no-mistakes-impact-");
  const link = join(dirname(directory), `${basename(directory)}-replacement`);
  const { assertOutputDirectory } = require("../planning-impact-artifacts-files");
  try {
    await symlink(directory, link);
    await assert.rejects(
      assertOutputDirectory({ path: link, identity: await stat(directory) }),
      /output directory path changed during planning artifact generation/,
    );
  } finally {
    await rm(link, { force: true });
    await rm(directory, { recursive: true, force: true });
  }
});

test("surfaces an output-path lookup error while restoring artifacts", async () => {
  const directory = await privateDirectory("no-mistakes-impact-");
  const manifest = join(directory, "changed-files.txt");
  const fs = require("node:fs/promises");
  const originalLstat = fs.lstat;
  const canonicalDirectory = await fs.realpath(directory);
  try {
    await writeFile(manifest, "a.mts\n");
    await withFsOverride(
      {
        lstat: async (path, ...args) => {
          if (path === canonicalDirectory) {
            const error = new Error("injected output-path lookup failure");
            error.code = "EACCES";
            throw error;
          }
          return originalLstat(path, ...args);
        },
      },
      async ({ writePlanningImpactArtifacts: writeArtifacts }) => {
        await assert.rejects(
          writeArtifacts(
            { root: "/repo", changedFilesManifest: manifest, outputDirectory: directory },
            async () => aggregateResult,
          ),
          (error) =>
            error instanceof AggregateError &&
            error.code === "EACCES" &&
            error.cause.code === "EACCES",
        );
      },
    );
  } finally {
    await rm(directory, { recursive: true, force: true });
    for (const entry of await readdir(dirname(canonicalDirectory))) {
      if (entry.startsWith(`.${basename(canonicalDirectory)}.planning-impact-`)) {
        await rm(join(dirname(canonicalDirectory), entry), { recursive: true, force: true });
      }
    }
  }
});

test("preserves concurrent public-path directory, file, and symlink victims", async () => {
  for (const kind of ["directory", "file", "symlink"]) {
    const directory = await privateDirectory(`no-mistakes-impact-${kind}-`);
    const outside = await privateDirectory(`no-mistakes-impact-${kind}-outside-`);
    const manifest = join(directory, "changed-files.txt");
    const victim = join(outside, "victim.txt");
    let parked;
    try {
      await writeFile(manifest, "a.mts\n");
      if (kind === "symlink") await writeFile(victim, "protected");
      await assert.rejects(
        writePlanningImpactArtifactsInternal(
          { root: "/repo", changedFilesManifest: manifest, outputDirectory: directory },
          async () => {
            assert.fail("analysis must not run when public restoration is occupied");
          },
          async (from, to) => {
            parked = from;
            if (kind === "directory") await mkdir(to);
            else if (kind === "file") await writeFile(to, "protected");
            else await symlink(victim, to);
            return false;
          },
        ),
        (error) => error.code === "EEXIST",
      );
      const metadata = await lstat(directory);
      if (kind === "directory") assert.equal(metadata.isDirectory(), true);
      else if (kind === "file") assert.equal(await readFile(directory, "utf8"), "protected");
      else {
        assert.equal(metadata.isSymbolicLink(), true);
        assert.equal(await readFile(victim, "utf8"), "protected");
      }
    } finally {
      await rm(directory, { recursive: true, force: true });
      await rm(parked, { recursive: true, force: true }).catch(() => {});
      await rm(outside, { recursive: true, force: true });
    }
  }
});

test("surfaces paired update and restoration failures with the parked recovery path", async () => {
  const directory = await privateDirectory("no-mistakes-impact-restore-failure-");
  const manifest = join(directory, "changed-files.txt");
  const fs = require("node:fs/promises");
  const originalOpen = fs.open;
  const originalRealpath = fs.realpath;
  const canonicalDirectory = await fs.realpath(directory);
  const updateError = new Error("injected status write failure");
  let parked;
  let renameCalls = 0;
  let restorationFailed = false;
  let outerFailureTransitions = 0;
  try {
    await writeFile(manifest, "a.mts\n");
    await withFsOverride(
      {
        open: async (path, ...args) => {
          if (String(path).includes(".dependencies.status.") && args[0] === "wx") {
            throw updateError;
          }
          return originalOpen(path, ...args);
        },
        realpath: async (path, ...args) => {
          if (restorationFailed && path === canonicalDirectory) {
            outerFailureTransitions += 1;
            throw new Error("outer failure transition attempted");
          }
          return originalRealpath(path, ...args);
        },
      },
      async ({ writePlanningImpactArtifacts: writeArtifacts }) => {
        await assert.rejects(
          writeArtifacts(
            { root: "/repo", changedFilesManifest: manifest, outputDirectory: directory },
            async () => {
              assert.fail("analysis must not run after the status update fails");
            },
          ),
          (error) => {
            assert.equal(error instanceof AggregateError, true);
            assert.equal(error.code, "EEXIST");
            assert.equal(error.cause.code, "EEXIST");
            assert.ok(error.message.includes(parked));
            assert.deepEqual(error.errors, [error.cause, updateError]);
            return true;
          },
        );
      },
      async (from, to) => {
        renameCalls += 1;
        parked = from;
        await writeFile(to, "concurrent public-path victim");
        restorationFailed = true;
        return false;
      },
    );
    assert.equal(renameCalls, 1);
    assert.equal(outerFailureTransitions, 0);
    assert.equal(await readFile(directory, "utf8"), "concurrent public-path victim");
    assert.equal((await lstat(parked)).isDirectory(), true);
  } finally {
    await rm(directory, { recursive: true, force: true });
    await rm(parked, { recursive: true, force: true }).catch(() => {});
  }
});

test("reports the public recovery path after restoration completed", async () => {
  const directory = await privateDirectory("no-mistakes-impact-restored-failure-");
  const manifest = join(directory, "changed-files.txt");
  const fs = require("node:fs/promises");
  const originalRealpath = fs.realpath;
  const canonicalDirectory = await fs.realpath(directory);
  let restored = false;
  let parked;
  try {
    await writeFile(manifest, "a.mts\n");
    await withFsOverride(
      {
        realpath: async (path, ...args) => {
          if (restored && path === canonicalDirectory) {
            const error = new Error("injected post-restoration lookup failure");
            error.code = "EIO";
            throw error;
          }
          return originalRealpath(path, ...args);
        },
      },
      async ({ writePlanningImpactArtifacts: writeArtifacts }) => {
        await assert.rejects(
          writeArtifacts(
            { root: "/repo", changedFilesManifest: manifest, outputDirectory: directory },
            async () => aggregateResult,
          ),
          (error) => {
            assert.equal(error instanceof AggregateError, true);
            assert.equal(error.code, "EIO");
            assert.match(error.message, /failed validation after restoration/u);
            assert.ok(error.message.includes(canonicalDirectory));
            assert.ok(!error.message.includes(parked));
            return true;
          },
        );
      },
      async (from, to) => {
        parked = from;
        await rename(from, to);
        restored = true;
        return true;
      },
    );
    assert.equal((await lstat(directory)).isDirectory(), true);
    await assert.rejects(lstat(parked), /ENOENT/u);
  } finally {
    restored = false;
    await rm(directory, { recursive: true, force: true });
    await rm(parked, { recursive: true, force: true }).catch(() => {});
  }
});

test("rejects staged symlink replacement instead of publishing it", async () => {
  const directory = await privateDirectory("no-mistakes-impact-");
  const manifest = join(directory, "changed-files.txt");
  const victim = join(directory, "victim.txt");
  const fs = require("node:fs/promises");
  const originalRename = fs.rename;
  try {
    await writeFile(manifest, "a.mts\n");
    await writeFile(victim, "protected");
    await withFsOverride(
      {
        rename: async (from, to) => {
          if (to.endsWith("dependencies.json")) {
            await unlink(from);
            await symlink(victim, from);
          }
          return originalRename(from, to);
        },
      },
      async ({ writePlanningImpactArtifacts: writeArtifacts }) => {
        await assert.rejects(
          writeArtifacts(
            { root: "/repo", changedFilesManifest: manifest, outputDirectory: directory },
            async () => aggregateResult,
          ),
          /published artifact changed during publication/,
        );
      },
    );
    assert.equal(await readFile(victim, "utf8"), "protected");
    await assert.rejects(lstat(join(directory, "dependencies.json")), /ENOENT/);
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

test("invalidates prior success statuses before analysis completes", async () => {
  const directory = await privateDirectory("no-mistakes-impact-");
  const manifest = join(directory, "changed-files.txt");
  let started;
  let finish;
  try {
    await writeFile(manifest, "a.mts\n");
    for (const name of ["dependencies", "dependents", "symbols", "plan"]) {
      await writeFile(join(directory, `${name}.status`), "0\n");
    }
    const analysisStarted = new Promise((resolveStarted) => {
      started = resolveStarted;
    });
    const analysisFinished = new Promise((resolveFinished) => {
      finish = resolveFinished;
    });
    const run = writePlanningImpactArtifacts(
      { root: "/repo", changedFilesManifest: manifest, outputDirectory: directory },
      async () => {
        started();
        await analysisFinished;
        return aggregateResult;
      },
    );
    await analysisStarted;
    for (const name of ["dependencies", "dependents", "symbols", "plan"]) {
      assert.equal(await readFile(join(directory, `${name}.status`), "utf8"), "1\n");
    }
    finish();
    await run;
    for (const name of ["dependencies", "dependents", "symbols", "plan"]) {
      assert.equal(await readFile(join(directory, `${name}.status`), "utf8"), "0\n");
    }
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

test("does not begin analysis when any success status cannot be invalidated", async () => {
  const directory = await privateDirectory("no-mistakes-impact-");
  const manifest = join(directory, "changed-files.txt");
  const fs = require("node:fs/promises");
  const originalRename = fs.rename;
  let failed = false;
  let analyzed = false;
  try {
    await writeFile(manifest, "a.mts\n");
    for (const name of ["dependencies", "dependents", "symbols", "plan"]) {
      await writeFile(join(directory, `${name}.status`), "0\n");
    }
    await withFsOverride(
      {
        rename: async (from, to) => {
          if (!failed && to.endsWith("symbols.status")) {
            failed = true;
            throw new Error("injected status invalidation failure");
          }
          return originalRename(from, to);
        },
      },
      async ({ writePlanningImpactArtifacts: writeArtifacts }) => {
        await assert.rejects(
          writeArtifacts(
            { root: "/repo", changedFilesManifest: manifest, outputDirectory: directory },
            async () => {
              analyzed = true;
              return aggregateResult;
            },
          ),
          /injected status invalidation failure/,
        );
      },
    );
    assert.equal(analyzed, false);
    for (const name of ["dependencies", "dependents", "symbols", "plan"]) {
      assert.equal(await readFile(join(directory, `${name}.status`), "utf8"), "1\n");
    }
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

test("rejects every reserved artifact destination as a changed-files manifest before invalidation", async () => {
  const directory = await privateDirectory("no-mistakes-impact-");
  let analyzed = false;
  try {
    for (const name of ["dependencies", "dependents", "symbols", "plan"]) {
      for (const extension of ["json", "stderr", "status"]) {
        const manifest = join(directory, `${name}.${extension}`);
        await writeFile(manifest, "a.mts\n");
        await assert.rejects(
          writePlanningImpactArtifacts(
            { root: "/repo", changedFilesManifest: manifest, outputDirectory: directory },
            async () => {
              analyzed = true;
              return aggregateResult;
            },
          ),
          /must not use a reserved artifact destination/,
        );
        assert.equal(await readFile(manifest, "utf8"), "a.mts\n");
        await unlink(manifest);
      }
    }

    // The lexical name remains reserved even when a symlink's target is safe.
    const target = join(directory, "changed-files.txt");
    const reservedAlias = join(directory, "plan.status");
    await writeFile(target, "a.mts\n");
    await symlink(target, reservedAlias);
    await assert.rejects(
      writePlanningImpactArtifacts(
        { root: "/repo", changedFilesManifest: reservedAlias, outputDirectory: directory },
        async () => {
          analyzed = true;
          return aggregateResult;
        },
      ),
      /must not use a reserved artifact destination/,
    );
    assert.equal((await lstat(reservedAlias)).isSymbolicLink(), true);
    assert.equal(await readFile(target, "utf8"), "a.mts\n");
    await unlink(reservedAlias);
    await unlink(target);

    // Retain the separate canonical check for a safe alias to a reserved target.
    const reservedTarget = join(directory, "plan.status");
    const safeAlias = join(directory, "changed-files.txt");
    await writeFile(reservedTarget, "a.mts\n");
    await symlink(reservedTarget, safeAlias);
    await assert.rejects(
      writePlanningImpactArtifacts(
        { root: "/repo", changedFilesManifest: safeAlias, outputDirectory: directory },
        async () => {
          analyzed = true;
          return aggregateResult;
        },
      ),
      /must not use a reserved artifact destination/,
    );
    assert.equal((await lstat(safeAlias)).isSymbolicLink(), true);
    assert.equal(await readFile(reservedTarget, "utf8"), "a.mts\n");
    assert.equal(analyzed, false);
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

test("rejects reserved artifact manifests through case-insensitive path aliases", async () => {
  const directory = await privateDirectory("no-mistakes-impact-");
  const manifest = join(directory, "PLAN.STATUS");
  const reservedAlias = join(directory, "plan.status");
  const fs = require("node:fs/promises");
  const originalStat = fs.stat;
  let analyzed = false;
  try {
    await writeFile(manifest, "a.mts\n");
    await withFsOverride(
      {
        stat: async (path, ...args) =>
          basename(path) === basename(reservedAlias)
            ? originalStat(manifest, ...args)
            : originalStat(path, ...args),
      },
      async ({ writePlanningImpactArtifacts: writeArtifacts }) => {
        await assert.rejects(
          writeArtifacts(
            { root: "/repo", changedFilesManifest: manifest, outputDirectory: directory },
            async () => {
              analyzed = true;
              return aggregateResult;
            },
          ),
          /must not use a reserved artifact destination/,
        );
      },
    );
    assert.equal(analyzed, false);
    assert.equal(await readFile(manifest, "utf8"), "a.mts\n");
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

test("does not reserve differently cased artifact names on case-sensitive filesystems", async () => {
  const directory = await privateDirectory("no-mistakes-impact-");
  const manifest = join(directory, "PLAN.STATUS");
  const reservedAlias = join(directory, "plan.status");
  const fs = require("node:fs/promises");
  const originalRealpath = fs.realpath;
  const originalStat = fs.stat;
  try {
    await writeFile(manifest, "README.md\n");
    await withFsOverride(
      {
        stat: async (path, ...args) => {
          if (basename(path) === basename(reservedAlias)) {
            const error = new Error("case-sensitive alias does not exist");
            error.code = "ENOENT";
            throw error;
          }
          return originalStat(path, ...args);
        },
      },
      async () => {
        const { isReservedArtifactPath } = require("../planning-impact-artifacts-inputs");
        assert.equal(await isReservedArtifactPath(await originalRealpath(manifest)), false);
      },
    );
    assert.equal(await readFile(manifest, "utf8"), "README.md\n");
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

test("best-effort failure reporting preserves the original directory error", async () => {
  const directory = await privateDirectory("no-mistakes-impact-");
  const manifest = join(directory, "changed-files.txt");
  try {
    await writeFile(manifest, "a.mts\n");
    await mkdir(join(directory, "dependents.json"));
    await assert.rejects(
      writePlanningImpactArtifacts(
        { root: "/repo", changedFilesManifest: manifest, outputDirectory: directory },
        async () => aggregateResult,
      ),
      /EISDIR/,
    );
    for (const name of ["dependencies", "dependents", "symbols", "plan"]) {
      assert.equal(await readFile(join(directory, `${name}.status`), "utf8"), "1\n");
      assert.ok((await readFile(join(directory, `${name}.stderr`), "utf8")).length > 0);
    }
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

test("attaches a failure-report restoration error and parked recovery path", async () => {
  const directory = await privateDirectory("no-mistakes-impact-report-restore-");
  const manifest = join(directory, "changed-files.txt");
  const analysisError = new Error("injected analysis failure");
  let parked;
  let renameCalls = 0;
  try {
    await writeFile(manifest, "a.mts\n");
    await assert.rejects(
      writePlanningImpactArtifactsInternal(
        { root: "/repo", changedFilesManifest: manifest, outputDirectory: directory },
        async () => {
          throw analysisError;
        },
        async (from, to) => {
          renameCalls += 1;
          if (renameCalls === 1) {
            await rename(from, to);
            return true;
          }
          parked = from;
          await writeFile(to, "concurrent public-path victim");
          return false;
        },
      ),
      (error) => {
        assert.equal(error, analysisError);
        assert.equal(error.failureReportingError instanceof AggregateError, true);
        assert.equal(error.failureReportingError.code, "EEXIST");
        assert.ok(error.failureReportingError.message.includes(parked));
        return true;
      },
    );
    assert.equal(renameCalls, 2);
    assert.equal(await readFile(directory, "utf8"), "concurrent public-path victim");
    assert.equal((await lstat(parked)).isDirectory(), true);
  } finally {
    await rm(directory, { recursive: true, force: true });
    await rm(parked, { recursive: true, force: true }).catch(() => {});
  }
});

test("aggregates restoration details when the original failure cannot be extended", () => {
  const restorationError = new Error("failure reporting stranded the directory");
  for (const originalError of [Object.freeze(new Error("frozen failure")), "primitive failure"]) {
    const result = preserveFailureReportingError(originalError, restorationError);
    assert.equal(result instanceof AggregateError, true);
    assert.deepEqual(result.errors, [originalError, restorationError]);
    assert.equal(result.cause, originalError);
    assert.match(result.message, /could not restore the output directory/u);
  }
});

test("best-effort failure reporting does not mask the original validation error", async () => {
  const directory = await privateDirectory("no-mistakes-impact-");
  const manifest = join(directory, "changed-files.txt");
  const fs = require("node:fs/promises");
  const originalRename = fs.rename;
  try {
    await writeFile(manifest, "../outside.mts\n");
    await withFsOverride(
      {
        rename: async (...args) => {
          if (args[0] === (await fs.realpath(directory))) {
            throw new Error("failure artifact publication is unavailable");
          }
          return originalRename(...args);
        },
      },
      async ({ writePlanningImpactArtifacts: writeArtifacts }) => {
        await assert.rejects(
          writeArtifacts(
            { root: "/repo", changedFilesManifest: manifest, outputDirectory: directory },
            async () => aggregateResult,
          ),
          /repository-relative/,
        );
      },
    );
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

test("uses identity checks when directory descriptors are unavailable", async () => {
  const directory = await privateDirectory("no-mistakes-impact-");
  const manifest = join(directory, "changed-files.txt");
  const fs = require("node:fs/promises");
  const originalOpen = fs.open;
  let directoryOpen = true;
  try {
    await writeFile(manifest, "a.mts\n");
    await withFsOverride(
      {
        open: async (path, ...args) => {
          if (directoryOpen && path === (await fs.realpath(directory))) {
            directoryOpen = false;
            const error = new Error("directory descriptors unavailable");
            error.code = "EISDIR";
            throw error;
          }
          return originalOpen(path, ...args);
        },
      },
      async ({ writePlanningImpactArtifacts: writeArtifacts }) => {
        const result = await writeArtifacts(
          { root: "/repo", changedFilesManifest: manifest, outputDirectory: directory },
          async () => aggregateResult,
        );
        assert.deepEqual(result.plan, aggregateResult.reports[3].result);
      },
    );
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

test("closes an output descriptor when its initial path identity changes", async () => {
  const directory = await privateDirectory("no-mistakes-impact-");
  const manifest = join(directory, "changed-files.txt");
  const fs = require("node:fs/promises");
  const originalOpen = fs.open;
  const originalStat = fs.stat;
  const canonicalDirectory = await fs.realpath(directory);
  let directoryStats = 0;
  let directoryClosed = false;
  try {
    await writeFile(manifest, "a.mts\n");
    await withFsOverride(
      {
        open: async (path, ...args) => {
          const handle = await originalOpen(path, ...args);
          if (path === canonicalDirectory) {
            const originalClose = handle.close.bind(handle);
            handle.close = async (...closeArgs) => {
              directoryClosed = true;
              return originalClose(...closeArgs);
            };
          }
          return handle;
        },
        stat: async (path, ...args) => {
          const metadata = await originalStat(path, ...args);
          if (path === canonicalDirectory) {
            directoryStats += 1;
            if (directoryStats === 2) {
              Object.defineProperty(metadata, "ino", { value: metadata.ino + 1 });
            }
          }
          return metadata;
        },
      },
      async ({ writePlanningImpactArtifacts: writeArtifacts }) => {
        await assert.rejects(
          writeArtifacts(
            { root: "/repo", changedFilesManifest: manifest, outputDirectory: directory },
            async () => aggregateResult,
          ),
          /output directory changed/,
        );
      },
    );
    assert.equal(directoryClosed, true);
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

test("rejects a changed output directory descriptor", async () => {
  const directory = await privateDirectory("no-mistakes-impact-");
  const manifest = join(directory, "changed-files.txt");
  const fs = require("node:fs/promises");
  const originalOpen = fs.open;
  try {
    await writeFile(manifest, "a.mts\n");
    const canonicalDirectory = await fs.realpath(directory);
    let wrapped = false;
    await withFsOverride(
      {
        open: async (path, ...args) => {
          const handle = await originalOpen(path, ...args);
          if (!wrapped && path === canonicalDirectory) {
            wrapped = true;
            const originalStat = handle.stat.bind(handle);
            let calls = 0;
            handle.stat = async () => {
              const metadata = await originalStat();
              calls += 1;
              if (calls >= 2) {
                Object.defineProperty(metadata, "ino", { value: metadata.ino + 1 });
              }
              return metadata;
            };
          }
          return handle;
        },
      },
      async ({ writePlanningImpactArtifacts: writeArtifacts }) => {
        await assert.rejects(
          writeArtifacts(
            { root: "/repo", changedFilesManifest: manifest, outputDirectory: directory },
            async () => aggregateResult,
          ),
          /output directory descriptor changed/,
        );
      },
    );
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

test("rejects a hardlinked staged artifact before publication", async () => {
  const directory = await privateDirectory("no-mistakes-impact-");
  const outside = await privateDirectory("no-mistakes-impact-outside-");
  const manifest = join(directory, "changed-files.txt");
  const outsideLink = join(outside, "staged-hardlink");
  const fs = require("node:fs/promises");
  const originalOpen = fs.open;
  try {
    await writeFile(manifest, "a.mts\n");
    await withFsOverride(
      {
        open: async (path, ...args) => {
          const handle = await originalOpen(path, ...args);
          if (path.includes(".dependencies.json.")) {
            const originalWriteFile = handle.writeFile.bind(handle);
            handle.writeFile = async (...writeArgs) => {
              const result = await originalWriteFile(...writeArgs);
              await link(path, outsideLink);
              return result;
            };
          }
          return handle;
        },
      },
      async ({ writePlanningImpactArtifacts: writeArtifacts }) => {
        await assert.rejects(
          writeArtifacts(
            { root: "/repo", changedFilesManifest: manifest, outputDirectory: directory },
            async () => aggregateResult,
          ),
          /staged artifact is not a private regular file: dependencies\.json/,
        );
      },
    );
    assert.equal(await readFile(outsideLink, "utf8"), '{"files":["a.mts"]}\n');
  } finally {
    await rm(directory, { recursive: true, force: true });
    await rm(outside, { recursive: true, force: true });
  }
});

test("rejects a staged pathname swap before publication", async () => {
  const directory = await privateDirectory("no-mistakes-impact-");
  const outside = await privateDirectory("no-mistakes-impact-outside-");
  const manifest = join(directory, "changed-files.txt");
  const victim = join(outside, "victim.txt");
  const fs = require("node:fs/promises");
  const originalLstat = fs.lstat;
  let swapped = false;
  try {
    await writeFile(manifest, "a.mts\n");
    await writeFile(victim, "protected");
    await withFsOverride(
      {
        lstat: async (path, ...args) => {
          if (!swapped && path.includes(".dependencies.json.")) {
            swapped = true;
            await unlink(path);
            await symlink(victim, path);
          }
          return originalLstat(path, ...args);
        },
      },
      async ({ writePlanningImpactArtifacts: writeArtifacts }) => {
        await assert.rejects(
          writeArtifacts(
            { root: "/repo", changedFilesManifest: manifest, outputDirectory: directory },
            async () => aggregateResult,
          ),
          // Coverage instrumentation can let either adjacent identity guard observe the swap.
          /(?:staged|published) artifact changed (?:before|during) publication: dependencies\.json/,
        );
      },
    );
    assert.equal(await readFile(victim, "utf8"), "protected");
  } finally {
    await rm(directory, { recursive: true, force: true });
    await rm(outside, { recursive: true, force: true });
  }
});

test("settles a failed content publish before recording failure statuses", async () => {
  const directory = await privateDirectory("no-mistakes-impact-");
  const manifest = join(directory, "changed-files.txt");
  const fs = require("node:fs/promises");
  const rename = fs.rename;
  try {
    await writeFile(manifest, "a.mts\n");
    await withFsOverride(
      {
        rename: async (from, to) => {
          if (to.endsWith("dependents.json")) throw new Error("injected publish failure");
          return rename(from, to);
        },
      },
      async ({ writePlanningImpactArtifacts: writeArtifacts }) => {
        await assert.rejects(
          writeArtifacts(
            { root: "/repo", changedFilesManifest: manifest, outputDirectory: directory },
            async () => aggregateResult,
          ),
          /injected publish failure/,
        );
      },
    );
    for (const name of ["dependencies", "dependents", "symbols", "plan"]) {
      assert.equal(await readFile(join(directory, `${name}.status`), "utf8"), "1\n");
      await assert.rejects(readFile(join(directory, `${name}.json`), "utf8"), /ENOENT/);
    }
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

test("continues failure diagnostics when report cleanup cannot stat a destination", async () => {
  const directory = await privateDirectory("no-mistakes-impact-");
  const manifest = join(directory, "changed-files.txt");
  const fs = require("node:fs/promises");
  const originalLstat = fs.lstat;
  const originalRename = fs.rename;
  try {
    await writeFile(manifest, "a.mts\n");
    await withFsOverride(
      {
        lstat: async (path, ...args) => {
          if (path.endsWith("dependencies.json")) {
            const error = new Error("injected artifact stat failure");
            error.code = "EIO";
            throw error;
          }
          return originalLstat(path, ...args);
        },
        rename: async (from, to) => {
          if (to.endsWith("dependencies.json")) throw new Error("injected publish failure");
          return originalRename(from, to);
        },
      },
      async ({ writePlanningImpactArtifacts: writeArtifacts }) => {
        await assert.rejects(
          writeArtifacts(
            { root: "/repo", changedFilesManifest: manifest, outputDirectory: directory },
            async () => aggregateResult,
          ),
          /injected publish failure/,
        );
      },
    );
    for (const name of ["dependencies", "dependents", "symbols", "plan"]) {
      assert.equal(await readFile(join(directory, `${name}.status`), "utf8"), "1\n");
      assert.ok((await readFile(join(directory, `${name}.stderr`), "utf8")).length > 0);
    }
    await assert.rejects(readFile(join(directory, "dependencies.json"), "utf8"), /ENOENT/);
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

test("requires a 0700 output directory and a manifest contained by it", async () => {
  const directory = await privateDirectory("no-mistakes-impact-");
  const outside = await privateDirectory("no-mistakes-impact-outside-");
  const nested = join(directory, "nested");
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
    await mkdir(nested);
    await writeFile(join(nested, "changed-files.txt"), "a.mts\n");
    await assert.rejects(
      writePlanningImpactArtifacts(
        {
          root: "/repo",
          changedFilesManifest: join(nested, "changed-files.txt"),
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
    await chmod(directory, 0o500);
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
    await chmod(directory, 0o700).catch(() => {});
    await rm(directory, { recursive: true, force: true });
    await rm(outside, { recursive: true, force: true });
  }
});

test("records a failed report schema as aggregate failure", async () => {
  const directory = await privateDirectory("no-mistakes-impact-");
  const manifest = join(directory, "changed-files.txt");
  try {
    await writeFile(manifest, "a.mts\n");
    await assert.rejects(
      writePlanningImpactArtifacts(
        { root: "/repo", changedFilesManifest: manifest, outputDirectory: directory },
        async () => ({
          reports: aggregateResult.reports.map((report) =>
            report.id === "dependencies" ? { ...report, type: "symbols" } : report,
          ),
        }),
      ),
      /dependencies type symbols; expected dependencies/,
    );
    assert.equal(await readFile(join(directory, "symbols.status"), "utf8"), "1\n");
  } finally {
    await rm(directory, { recursive: true, force: true });
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
      acquirePlanningArtifactLock: async () => 1,
      releasePlanningArtifactLock: async () => {},
      renameNoReplace: async (from, to) => {
        await rename(from, to);
        return true;
      },
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

test("public artifact API preserves a concurrent public-path victim", async () => {
  const directory = await privateDirectory("no-mistakes-impact-public-race-");
  const manifest = join(directory, "changed-files.txt");
  const previous = require.extensions[".node"];
  let parked;
  delete require.cache[require.resolve(indexPath)];
  delete require.cache[require.resolve(addonPath)];
  require.extensions[".node"] = (module, filename) => {
    assert.equal(filename, addonPath);
    module.exports = {
      analyzeProjectJson: async () => {
        assert.fail("analysis must not run when public restoration is occupied");
      },
      acquirePlanningArtifactLock: async () => 1,
      releasePlanningArtifactLock: async () => {},
      renameNoReplace: async (from, to) => {
        parked = from;
        await writeFile(to, "protected");
        return false;
      },
    };
  };
  try {
    await writeFile(manifest, "a.mts\n");
    const api = require(indexPath);
    await assert.rejects(
      api.writePlanningImpactArtifacts({
        root: "/repo",
        changedFilesManifest: manifest,
        outputDirectory: directory,
      }),
      (error) => error.code === "EEXIST",
    );
    assert.equal(await readFile(directory, "utf8"), "protected");
  } finally {
    delete require.cache[require.resolve(indexPath)];
    delete require.cache[require.resolve(addonPath)];
    if (previous) require.extensions[".node"] = previous;
    else delete require.extensions[".node"];
    await rm(directory, { recursive: true, force: true });
    await rm(parked, { recursive: true, force: true }).catch(() => {});
  }
});

test("public artifact API preserves a manifest that collides with a reserved destination", async () => {
  const directory = await privateDirectory("no-mistakes-impact-");
  const manifest = join(directory, "plan.status");
  const previous = require.extensions[".node"];
  delete require.cache[require.resolve(indexPath)];
  delete require.cache[require.resolve(addonPath)];
  require.extensions[".node"] = (module, filename) => {
    assert.equal(filename, addonPath);
    module.exports = {
      analyzeProjectJson: async () => {
        assert.fail("reserved manifest collision must fail before public analysis");
      },
      acquirePlanningArtifactLock: async () => 1,
      releasePlanningArtifactLock: async () => {},
    };
  };
  try {
    await writeFile(manifest, "a.mts\n");
    const api = require(indexPath);
    await assert.rejects(
      api.writePlanningImpactArtifacts({
        root: "/repo",
        changedFilesManifest: manifest,
        outputDirectory: directory,
      }),
      /must not use a reserved artifact destination/,
    );
    assert.equal(await readFile(manifest, "utf8"), "a.mts\n");
  } finally {
    delete require.cache[require.resolve(indexPath)];
    delete require.cache[require.resolve(addonPath)];
    if (previous) require.extensions[".node"] = previous;
    else delete require.extensions[".node"];
    await rm(directory, { recursive: true, force: true });
  }
});

test("exports the artifact writer through the real ESM entrypoint", async () => {
  const directory = await privateDirectory("no-mistakes-impact-");
  const manifest = join(directory, "changed-files.txt");
  const previous = require.extensions[".node"];
  delete require.cache[require.resolve(indexPath)];
  delete require.cache[require.resolve(addonPath)];
  require.extensions[".node"] = (module, filename) => {
    assert.equal(filename, addonPath);
    module.exports = {
      analyzeProjectJson: async () => JSON.stringify(aggregateResult),
      acquirePlanningArtifactLock: async () => 1,
      releasePlanningArtifactLock: async () => {},
      renameNoReplace: async (from, to) => {
        await rename(from, to);
        return true;
      },
    };
  };
  try {
    await writeFile(manifest, "a.mts\n");
    const api = await import(`${pathToFileURL(esmIndexPath).href}?impact=${Date.now()}`);
    const result = await api.writePlanningImpactArtifacts({
      root: "/repo",
      changedFilesManifest: manifest,
      outputDirectory: directory,
    });
    assert.deepEqual(result.dependents, { files: [] });
  } finally {
    delete require.cache[require.resolve(indexPath)];
    delete require.cache[require.resolve(addonPath)];
    if (previous) require.extensions[".node"] = previous;
    else delete require.extensions[".node"];
    await rm(directory, { recursive: true, force: true });
  }
});

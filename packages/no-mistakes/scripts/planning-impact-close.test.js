const assert = require("node:assert/strict");
const test = globalThis.test || require("node:test").test;
const { chmod, mkdtemp, readdir, rm, writeFile } = require("node:fs/promises");
const { tmpdir } = require("node:os");
const { join } = require("node:path");

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

async function withFsOverride(overrides, callback) {
  const fs = require("node:fs/promises");
  const originals = Object.fromEntries(Object.keys(overrides).map((name) => [name, fs[name]]));
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
      writePlanningImpactArtifacts: async (options, analyzeProject) =>
        artifacts.writePlanningImpactArtifacts(options, analyzeProject, async (from, to) => {
          await fs.rename(from, to);
          return true;
        }),
    });
  } finally {
    Object.assign(fs, originals);
    for (const helperModule of helperModules) delete require.cache[require.resolve(helperModule)];
  }
}

test("removes a staged artifact when closing the staged handle fails", async () => {
  const directory = await privateDirectory("no-mistakes-impact-close-");
  const manifest = join(directory, "changed-files.txt");
  const fs = require("node:fs/promises");
  const originalOpen = fs.open;
  try {
    await writeFile(manifest, "a.mts\n");
    await withFsOverride(
      {
        open: async (path, ...args) => {
          const handle = await originalOpen(path, ...args);
          if (String(path).includes(".dependencies.json.")) {
            const originalClose = handle.close.bind(handle);
            handle.close = async () => {
              await originalClose();
              const error = new Error("injected close failure");
              error.code = "EIO";
              throw error;
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
          /injected close failure/,
        );
      },
    );
    const leftovers = (await readdir(directory)).filter(
      (name) => name.includes(".tmp") || name.startsWith(".dependencies"),
    );
    assert.deepEqual(leftovers, []);
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

test("closes the output handle when directory validation fails after open", async () => {
  const directory = await privateDirectory("no-mistakes-impact-dir-close-");
  const manifest = join(directory, "changed-files.txt");
  const fs = require("node:fs/promises");
  const originalOpen = fs.open;
  try {
    await writeFile(manifest, "a.mts\n");
    await withFsOverride(
      {
        open: async (path, ...args) => {
          const handle = await originalOpen(path, ...args);
          if (path === directory && args[0] === "r") {
            await chmod(directory, 0o755);
            const originalClose = handle.close.bind(handle);
            handle.close = async () => {
              await originalClose();
              const error = new Error("injected directory close failure");
              error.code = "EIO";
              throw error;
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
          /output directory changed during planning artifact generation/,
        );
      },
    );
  } finally {
    await chmod(directory, 0o700).catch(() => {});
    await rm(directory, { recursive: true, force: true });
  }
});

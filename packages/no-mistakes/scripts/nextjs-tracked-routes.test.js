const assert = require("node:assert/strict");
const { execFileSync } = require("node:child_process");
const { cp, mkdtemp, rename, rm } = require("node:fs/promises");
const { tmpdir } = require("node:os");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");

function git(root, args) {
  const env = { ...process.env };
  for (const key of ["GIT_DIR", "GIT_COMMON_DIR", "GIT_INDEX_FILE", "GIT_WORK_TREE"])
    delete env[key];
  execFileSync("git", ["-C", root, ...args], { env, stdio: "pipe" });
}

test(
  "compiled CJS and ESM checks share tracked redirect route semantics",
  { skip: !compiled },
  async () => {
    const root = await mkdtemp(join(tmpdir(), "no-mistakes-tracked-routes-"));
    try {
      await cp(
        join(
          __dirname,
          "../../../test-cases/rules/nextjs-redirect-destinations/fixture/tracked-routes",
        ),
        root,
        { recursive: true },
      );
      await rename(join(root, ".gitignore.fixture"), join(root, ".gitignore"));
      git(root, ["init", "-q", "--initial-branch=main"]);
      git(root, [
        "add",
        ".gitignore",
        ".no-mistakes.yml",
        ".filesystem.yml",
        "next.config.ts",
        "app/tracked",
        "app/(group)",
        "app/posts",
        "app/docs",
        "app/optional",
      ]);
      const api = require("../index.js");
      const esm = await import("../index.mjs");
      const options = { root, config: join(root, ".no-mistakes.yml") };
      const pending = api.check(options);
      assert.equal(typeof pending.then, "function");
      const report = await pending;
      assert.equal(report.rules.length, 6);
      assert.deepEqual(await esm.check(options), report);
      const filesystem = await api.check({ root, config: join(root, ".filesystem.yml") });
      assert.equal(filesystem.rules.length, 3);
      git(root, ["add", "-f", "app/untracked", "app/ignored"]);
      assert.deepEqual((await api.check(options)).rules, []);
      assert.deepEqual((await esm.check(options)).rules, []);
    } finally {
      await rm(root, { recursive: true, force: true });
    }
  },
);

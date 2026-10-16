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

for (const fixtureName of ["nested-projects", "nested-ignored-config"])
  test(
    `compiled CJS and ESM checks use ${fixtureName} indexes under non-Git roots`,
    { skip: !compiled },
    async () => {
      const root = await mkdtemp(join(tmpdir(), "no-mistakes-nested-tracked-routes-"));
      try {
        await cp(
          join(
            __dirname,
            "../../../test-cases/rules/nextjs-redirect-destinations/fixture",
            fixtureName,
          ),
          root,
          { recursive: true },
        );
        const project = join(root, "packages/web");
        await rename(join(project, ".gitignore.fixture"), join(project, ".gitignore"));
        git(project, ["init", "-q", "--initial-branch=main"]);
        // The umbrella walk omits these ignored pages; the nested Git index owns them.
        git(project, ["add", "-f", "."]);
        git(project, ["rm", "--cached", "-r", "app/untracked"]);
        const api = require("../index.js");
        const esm = await import("../index.mjs");
        const options = { root, config: join(root, ".no-mistakes.yml") };
        const report = await api.check(options);
        assert.equal(report.rules.length, 3);
        assert.ok(report.rules.every((finding) => finding.message.includes("'/untracked")));
        assert.deepEqual(await esm.check(options), report);
        git(project, ["add", "app/untracked"]);
        assert.deepEqual((await api.check(options)).rules, []);
        assert.deepEqual((await esm.check(options)).rules, []);
        git(project, ["update-index", "--skip-worktree", "app/tracked/page.tsx"]);
        await rm(join(project, "app/tracked/page.tsx"));
        await rm(join(project, "app/untracked/page.tsx"));
        // Index membership survives sparse and unstaged worktree removals.
        assert.deepEqual((await api.check(options)).rules, []);
        assert.deepEqual((await esm.check(options)).rules, []);
        git(project, ["update-index", "--force-remove", "app/tracked/page.tsx"]);
        assert.equal((await api.check(options)).rules.length, 1);
      } finally {
        await rm(root, { recursive: true, force: true });
      }
    },
  );

test(
  "compiled CJS and ESM explain how to recover a missing tracked inventory",
  { skip: !compiled },
  async () => {
    const root = await mkdtemp(join(tmpdir(), "no-mistakes-no-git-tracked-routes-"));
    try {
      await cp(
        join(
          __dirname,
          "../../../test-cases/rules/nextjs-redirect-destinations/fixture/tracked-routes",
        ),
        root,
        { recursive: true },
      );
      const api = require("../index.js");
      const esm = await import("../index.mjs");
      for (const facade of [api, esm]) {
        await assert.rejects(
          facade.check({ root, config: join(root, ".no-mistakes.yml") }),
          (error) => {
            for (const expected of [
              "fails closed",
              "Git-backed configured project",
              "run_filesystem_rules_with_files()",
              "trackedRoutesOnly: false",
            ])
              assert.ok(error.message.includes(expected), error.message);
            return true;
          },
        );
      }
    } finally {
      await rm(root, { recursive: true, force: true });
    }
  },
);

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

// Keep each fixture state independent so the native duration gate measures one
// CJS/ESM parity scenario rather than several consecutive analysis requests.
for (const scenario of ["untracked", "filesystem", "force-added"])
  test(
    `compiled CJS and ESM checks share tracked redirect route semantics: ${scenario}`,
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
        if (scenario === "force-added") git(root, ["add", "-f", "app/untracked", "app/ignored"]);
        const config = scenario === "filesystem" ? ".filesystem.yml" : ".no-mistakes.yml";
        const options = { root, config: join(root, config) };
        const pending = api.check(options);
        assert.equal(typeof pending.then, "function");
        const report = await pending;
        assert.equal(
          report.rules.length,
          scenario === "untracked" ? 6 : scenario === "filesystem" ? 3 : 0,
        );
        assert.deepEqual(await esm.check(options), report);
      } finally {
        await rm(root, { recursive: true, force: true });
      }
    },
  );

for (const fixtureName of ["nested-projects", "nested-ignored-config"])
  for (const scenario of ["untracked", "added", "sparse", "index-removed"])
    test(
      `compiled CJS and ESM checks use ${fixtureName} indexes under non-Git roots: ${scenario}`,
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
          if (scenario !== "untracked") git(project, ["add", "app/untracked"]);
          if (scenario === "sparse" || scenario === "index-removed") {
            git(project, ["update-index", "--skip-worktree", "app/tracked/page.tsx"]);
            await rm(join(project, "app/tracked/page.tsx"));
            await rm(join(project, "app/untracked/page.tsx"));
            // Index membership survives sparse and unstaged worktree removals.
          }
          if (scenario === "index-removed")
            git(project, ["update-index", "--force-remove", "app/tracked/page.tsx"]);
          const api = require("../index.js");
          const esm = await import("../index.mjs");
          const options = { root, config: join(root, ".no-mistakes.yml") };
          const report = await api.check(options);
          assert.equal(
            report.rules.length,
            scenario === "untracked" ? 3 : scenario === "index-removed" ? 1 : 0,
          );
          if (scenario === "untracked")
            assert.ok(report.rules.every((finding) => finding.message.includes("'/untracked")));
          assert.deepEqual(await esm.check(options), report);
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

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

async function withTrackedFixture(fixtureName, run) {
  const root = await mkdtemp(join(tmpdir(), "no-mistakes-tracked-routes-"));
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
    const project = fixtureName === "tracked-routes" ? root : join(root, "packages/web");
    await rename(join(project, ".gitignore.fixture"), join(project, ".gitignore"));
    git(project, ["init", "-q", "--initial-branch=main"]);
    if (fixtureName === "tracked-routes") {
      git(project, [
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
    } else {
      // The umbrella walk omits these ignored pages; the nested Git index owns them.
      git(project, ["add", "-f", "."]);
      git(project, ["rm", "--cached", "-r", "app/untracked"]);
    }
    await run(root, project);
  } finally {
    await rm(root, { recursive: true, force: true });
  }
}

async function removeWorktreePages(project) {
  git(project, ["update-index", "--skip-worktree", "app/tracked/page.tsx"]);
  await rm(join(project, "app/tracked/page.tsx"));
  await rm(join(project, "app/untracked/page.tsx"));
}

// Each test makes at most two native requests. Independent fixture states keep
// facade parity bounded; transition cases below also guard same-root invalidation.
for (const scenario of ["untracked", "filesystem", "force-added"])
  test(
    `compiled CJS and ESM checks share tracked redirect route semantics: ${scenario}`,
    { skip: !compiled },
    () =>
      withTrackedFixture("tracked-routes", async (root) => {
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
      }),
  );

for (const fixtureName of ["nested-projects", "nested-ignored-config"])
  for (const scenario of ["untracked", "added", "sparse", "index-removed"])
    test(
      `compiled CJS and ESM checks use ${fixtureName} indexes under non-Git roots: ${scenario}`,
      { skip: !compiled },
      () =>
        withTrackedFixture(fixtureName, async (root, project) => {
          if (scenario !== "untracked") git(project, ["add", "app/untracked"]);
          if (scenario === "sparse" || scenario === "index-removed")
            await removeWorktreePages(project);
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
        }),
    );

for (const facadeName of ["CJS", "ESM"]) {
  for (const transition of ["filesystem", "force-added"])
    test(
      `compiled ${facadeName} tracked routes refresh after ${transition}`,
      { skip: !compiled },
      () =>
        withTrackedFixture("tracked-routes", async (root) => {
          const api = facadeName === "CJS" ? require("../index.js") : await import("../index.mjs");
          const options = { root, config: join(root, ".no-mistakes.yml") };
          assert.equal((await api.check(options)).rules.length, 6);
          if (transition === "filesystem") options.config = join(root, ".filesystem.yml");
          else git(root, ["add", "-f", "app/untracked", "app/ignored"]);
          // Both requests use the same root and facade; stale index/config state fails.
          assert.equal(
            (await api.check(options)).rules.length,
            transition === "filesystem" ? 3 : 0,
          );
        }),
    );

  for (const fixtureName of ["nested-projects", "nested-ignored-config"])
    for (const transition of ["added", "sparse", "index-removed"])
      test(
        `compiled ${facadeName} ${fixtureName} refreshes after ${transition}`,
        { skip: !compiled },
        () =>
          withTrackedFixture(fixtureName, async (root, project) => {
            if (transition !== "added") git(project, ["add", "app/untracked"]);
            if (transition === "index-removed") await removeWorktreePages(project);
            const api =
              facadeName === "CJS" ? require("../index.js") : await import("../index.mjs");
            const options = { root, config: join(root, ".no-mistakes.yml") };
            assert.equal((await api.check(options)).rules.length, transition === "added" ? 3 : 0);
            if (transition === "added") git(project, ["add", "app/untracked"]);
            else if (transition === "sparse") await removeWorktreePages(project);
            else git(project, ["update-index", "--force-remove", "app/tracked/page.tsx"]);
            // A warm catalog must observe changes without sharing state across tests.
            assert.equal(
              (await api.check(options)).rules.length,
              transition === "index-removed" ? 1 : 0,
            );
          }),
      );
}

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

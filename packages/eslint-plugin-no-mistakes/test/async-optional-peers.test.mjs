import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { cpSync, existsSync, mkdtempSync, readFileSync, realpathSync, rmSync } from "node:fs";
import { createRequire, Module } from "node:module";
import { tmpdir } from "node:os";
import { delimiter, dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { Linter } from "eslint";
import { it, vi } from "vitest";
import { plugin } from "./helpers.mjs";

const require = createRequire(import.meta.url);
const packageRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const fixture = resolve(
  packageRoot,
  "../../test-cases/eslint-plugin/async-call-disposition/fixture/optional-peers",
);
function run(command, args, cwd) {
  const result = spawnSync(command, args, {
    cwd,
    encoding: "utf8",
    env: { ...process.env, NODE_PATH: "", NODE_OPTIONS: "" },
  });
  assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
  return result.stdout;
}

function runNpm(args, cwd) {
  // pnpm's Node runtime may not bundle npm. Use PATH, including POSIX shims.
  if (process.platform !== "win32") return run("npm", args, cwd);
  // Windows cannot spawn .cmd shims directly; run that installation's JS entry.
  const command = process.env.PATH.split(delimiter)
    .map((directory) => join(directory, "npm.cmd"))
    .find(existsSync);
  assert.ok(command, "consumer installation tests require npm on PATH");
  const cli = join(dirname(realpathSync(command)), "node_modules/npm/bin/npm-cli.js");
  return run(process.execPath, [cli, ...args], cwd);
}

it("loads a packed plugin and conservatively lints consumers with optional proof peers absent", () => {
  // Physical installation outside the repository prevents workspace peers from
  // accidentally making an absent-peer consumer appear to work.
  const root = mkdtempSync(join(tmpdir(), "no-mistakes-optional-peers-"));
  try {
    const packed = JSON.parse(
      runNpm(
        [
          "pack",
          "--offline",
          "--ignore-scripts",
          "--json",
          "--pack-destination",
          root,
          "--cache",
          join(root, "npm-cache"),
        ],
        packageRoot,
      ),
    )[0];
    cpSync(fixture, root, { recursive: true });
    runNpm(
      [
        "install",
        "--offline",
        "--ignore-scripts",
        "--legacy-peer-deps",
        "--no-audit",
        "--no-fund",
        "--cache",
        join(root, "npm-cache"),
        join(root, packed.filename),
      ],
      root,
    );
    const manifest = JSON.parse(
      readFileSync(join(root, "node_modules/eslint-plugin-no-mistakes/package.json"), "utf8"),
    );
    for (const peer of ["typescript", "@typescript-eslint/parser"]) {
      assert.equal(manifest.dependencies?.[peer], undefined);
      assert.equal(typeof manifest.peerDependencies[peer], "string");
      assert.equal(manifest.peerDependenciesMeta[peer].optional, true);
    }
    const runner = join(root, "run.cjs");
    const eslint = require.resolve("eslint");
    run(process.execPath, [runner, eslint, "none"], root);
    // Copy the actual package rather than symlinking: symlinks could expose
    // the workspace parser through Node's realpath-based module resolution.
    cpSync(
      dirname(require.resolve("typescript/package.json")),
      join(root, "node_modules/typescript"),
      { recursive: true },
    );
    run(process.execPath, [runner, eslint, "typescript"], root);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}, 30_000);

it.each(["typescript", "@typescript-eslint/parser"])(
  "reports unproven imported contracts when %s cannot load",
  (missing) => {
    const load = Module._load;
    const peer = vi.spyOn(Module, "_load").mockImplementation(function (name, ...args) {
      if (name === missing) throw new Error("Optional peer failed to load");
      return load.call(this, name, ...args);
    });
    try {
      const filename = join(fixture, "imported.js");
      const messages = new Linter({ cwd: fixture }).verify(
        readFileSync(filename, "utf8"),
        {
          plugins: { "no-mistakes": plugin },
          rules: {
            "no-mistakes/async-call-disposition": [
              "error",
              {
                targets: [
                  { sourceSpecifierPatterns: ["@app/jobs"], calleeNamePatterns: ["enqueueEmail"] },
                ],
              },
            ],
          },
        },
        { filename },
      );
      assert.deepEqual(
        messages.map(({ line, messageId }) => [line, messageId]),
        [4, 6, 8].map((line) => [line, "disposition"]),
      );
    } finally {
      peer.mockRestore();
    }
  },
);

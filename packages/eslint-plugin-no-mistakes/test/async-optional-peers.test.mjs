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
function run(command, args, cwd, status = 0, env = {}) {
  const result = spawnSync(command, args, {
    cwd,
    encoding: "utf8",
    env: { ...process.env, NODE_PATH: "", NODE_OPTIONS: "", ...env },
  });
  assert.equal(result.status, status, `${result.stdout}\n${result.stderr}`);
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

it("loads a packed plugin with TypeScript-only, absent, and incompatible proof peers", () => {
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
    const checkerRequire = createRequire(require.resolve("oxlint-tsgolint/package.json"));
    const checker = checkerRequire.resolve(
      `@oxlint-tsgolint/${process.platform}-${process.arch}/tsgolint${process.platform === "win32" ? ".exe" : ""}`,
    );
    const oxlint = join(dirname(require.resolve("oxlint")), "../bin/oxlint");
    const oxlintOutput = run(
      process.execPath,
      [
        oxlint,
        "--type-aware",
        "--deny-warnings",
        "--format",
        "json",
        "--config",
        join(root, ".oxlintrc.json"),
        join(root, "imported.js"),
      ],
      root,
      1,
      { OXLINT_TSGOLINT_PATH: checker },
    );
    assert.ok(oxlintOutput.startsWith("{"), oxlintOutput);
    const diagnostics = JSON.parse(oxlintOutput).diagnostics;
    assert.deepEqual(
      diagnostics.map((item) => [item.labels[0].span.line, item.code]),
      [4, 6, 8].map((line) => [line, "no-mistakes(async-call-disposition)"]),
    );
    // Copy the actual package rather than symlinking: symlinks could expose
    // the workspace parser through Node's realpath-based module resolution.
    cpSync(
      dirname(require.resolve("typescript/package.json")),
      join(root, "node_modules/typescript"),
      { recursive: true },
    );
    const typedOutput = run(
      process.execPath,
      [
        oxlint,
        "--type-aware",
        "--deny-warnings",
        "--format",
        "json",
        "--config",
        join(root, ".oxlintrc.json"),
        join(root, "imported.js"),
        join(root, "recall.js"),
        join(root, "syntax.js"),
        join(root, "heritage.js"),
        join(root, "type-shadow.js"),
        join(root, "global-value.js"),
        ...[
          "global-type.js",
          "global-type-import.js",
          "global-type-specifier.js",
          "global-type-local.ts",
        ].map((file) => join(root, file)),
        join(root, "unsupported.js"),
        join(root, "unsupported-enum.js"),
      ],
      root,
      1,
      { OXLINT_TSGOLINT_PATH: checker },
    );
    assert.deepEqual(
      JSON.parse(typedOutput)
        .diagnostics.map((item) => [
          item.filename.split(/[\\/]/).at(-1),
          item.labels[0].span.line,
          item.code,
        ])
        .sort(),
      [
        ...[3, 5, 7, 9, 12, 15].map((line) => [
          "recall.js",
          line,
          "no-mistakes(async-call-disposition)",
        ]),
        ["unsupported.js", 3, "no-mistakes(async-call-disposition)"],
        ["unsupported-enum.js", 3, "no-mistakes(async-call-disposition)"],
        ...["global-type.js", "global-type-import.js", "global-type-specifier.js"].map((file) => [
          file,
          3,
          "no-mistakes(async-call-disposition)",
        ]),
        ["global-type-local.ts", 9, "no-mistakes(async-call-disposition)"],
        ...[4, 6, 8, 12].map((line) => [
          "type-shadow.js",
          line,
          "no-mistakes(async-call-disposition)",
        ]),
      ].sort(),
    );
    run(process.execPath, [runner, eslint, "typescript"], root);
    rmSync(join(root, "node_modules/typescript"), { recursive: true });
    // Stub versions exercise peer metadata without network access. Their empty
    // exports also prove incompatible optional tooling cannot crash linting.
    // They isolate this plugin's constraints, not the parser's own peer ranges.
    runNpm(
      [
        "install",
        "--offline",
        "--ignore-scripts",
        "--no-audit",
        "--no-fund",
        "--cache",
        join(root, "npm-cache"),
        join(root, packed.filename),
        ...["typescript", "parser", "eslint"].map((name) => join(root, "peer-stubs", name)),
      ],
      root,
    );
    run(process.execPath, [runner, eslint, "incompatible"], root);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}, 30_000);

it.each([
  "typescript",
  "@typescript-eslint/parser",
  "legacy TypeScript",
  "empty TypeScript",
  "missing resolution cache",
  "empty parser",
])("reports unproven imported contracts with unavailable proof tooling (%s)", (missing) => {
  const load = Module._load;
  const peer = vi.spyOn(Module, "_load").mockImplementation(function (name, ...args) {
    if (name === missing) throw new Error("Optional peer failed to load");
    if (name === "typescript" && missing === "empty TypeScript") return {};
    if (name === "@typescript-eslint/parser" && missing === "empty parser") return {};
    if (name === "typescript" && missing === "missing resolution cache") {
      return { ...load.call(this, name, ...args), createModuleResolutionCache: undefined };
    }
    if (name === "typescript" && missing === "legacy TypeScript") {
      const ts = load.call(this, name, ...args);
      // TypeScript 4.8 has no Bundler mode; its legacy resolver is not equivalent.
      return { ...ts, ModuleResolutionKind: { ...ts.ModuleResolutionKind, Bundler: undefined } };
    }
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
});

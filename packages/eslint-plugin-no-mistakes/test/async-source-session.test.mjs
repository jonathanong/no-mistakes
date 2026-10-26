import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { Linter } from "eslint";
import { it, vi } from "vitest";
import { createRequire, Module } from "node:module";
import { basename } from "node:path";
import { plugin } from "./helpers.mjs";
import {
  cli,
  fixtureFile,
  fixtureText,
  instrumentSources,
  require,
  sessionFixture,
  targets,
  tsParser,
  withArgv,
} from "./async-source-session-shared.mjs";

const { currentSources } = require("../src/rules/async-source-files");
const { createImportedSources } = require("../src/rules/async-source-store");
const { sourceOwner, withSourceOwner } = require("../src/rules/async-source-owner");

it.each([false, true])(
  "shares source preparation in a real multi-file Oxlint invocation (typed=%s)",
  (typed) => {
    const files = Array.from({ length: 8 }, (_, i) => fixtureFile(`callers/caller${i}.ts`));
    const checkerRequire = createRequire(require.resolve("oxlint-tsgolint/package.json"));
    const checker = checkerRequire.resolve(
      `@oxlint-tsgolint/${process.platform}-${process.arch}/tsgolint${process.platform === "win32" ? ".exe" : ""}`,
    );
    const result = spawnSync(
      process.execPath,
      [
        "--require",
        fixtureFile("profile.cjs"),
        cli,
        ...(typed ? ["--type-aware"] : []),
        "--deny-warnings",
        "--threads",
        "1",
        "--config",
        fixtureFile(".oxlintrc.json"),
        "--format",
        "json",
        ...files,
      ],
      {
        cwd: sessionFixture,
        encoding: "utf8",
        env: { ...process.env, NODE_OPTIONS: "", OXLINT_TSGOLINT_PATH: checker },
      },
    );
    assert.equal(result.status, 1, result.stdout + result.stderr);
    const diagnostics = JSON.parse(result.stdout).diagnostics;
    assert.deepEqual(
      diagnostics
        .map((item) => [basename(item.filename), item.labels[0].span.line, item.code])
        .sort(),
      files
        .map((filename) => [basename(filename), 3, "no-mistakes(async-call-disposition)"])
        .sort(),
    );
    const profiles = result.stderr
      .split(/\r?\n/)
      .filter((line) => line.startsWith("SOURCE_SESSION_COUNTS "));
    assert.equal(profiles.length, 1, result.stderr);
    const stats = JSON.parse(profiles[0].slice("SOURCE_SESSION_COUNTS ".length));
    // Equality of diagnostics alone cannot prove source preparation was shared.
    assert.equal(stats.parses, 2);
    assert.deepEqual(
      stats.reads,
      Object.fromEntries(
        ["tsconfig.json", "shared/jobs.ts", "shared/consumers.ts"].map((name) => [
          fixtureFile(name),
          1,
        ]),
      ),
    );
    assert.equal(stats.resolutions, files.length * 2);
  },
);

it("keeps repeated ordinary ESLint verifications fresh", () => {
  const probe = instrumentSources();
  try {
    const filename = fixtureFile("callers/caller0.ts");
    const linter = new Linter({ cwd: sessionFixture });
    const config = {
      files: ["**/*.ts"],
      languageOptions: { parser: tsParser },
      plugins: { "no-mistakes": plugin },
      rules: { "no-mistakes/async-call-disposition": ["error", { targets }] },
    };
    for (let run = 0; run < 2; run++) {
      const findings = linter.verify(fixtureText("callers/caller0.ts"), config, { filename });
      assert.deepEqual(
        findings.map(({ line, messageId }) => [line, messageId]),
        [[3, "disposition"]],
      );
      assert.equal(probe.stats.parses, (run + 1) * 2);
    }
    assert.deepEqual([...probe.stats.reads.values()], [2, 2, 2]);
  } finally {
    probe.restore();
  }
});

it.each([
  [cli, ["--lsp"]],
  [cli, ["--fix"]],
  [fixtureFile("unknown-host/bin/host.cjs"), []],
])("keeps successive file proofs fresh for host %s %j", (entry, args) => {
  const probe = instrumentSources();
  try {
    withArgv(entry, args, () => {
      const context = { filename: fixtureFile("callers/caller0.ts") };
      const parsed = [];
      const hooks = withSourceOwner({
        create() {
          assert.equal(sourceOwner(context), null);
          const sources = createImportedSources(context);
          const job = sources.resolve("@session/jobs", context.filename);
          parsed.push(sources.load(job));
          return { Program() {} };
        },
      }).createOnce(context);
      for (let file = 0; file < 2; file++) {
        hooks.before();
        hooks.after();
      }
      assert.notEqual(parsed[0].ast, parsed[1].ast);
    });
    assert.equal(probe.stats.parses, 2);
    assert.deepEqual([...probe.stats.reads.values()], [2, 2]);
  } finally {
    probe.restore();
  }
});

it("keeps mutable host ASTs in a per-file overlay", () => {
  const firstAst = tsParser.parseForESLint(fixtureText("callers/caller0.ts")).ast;
  const nextAst = tsParser.parseForESLint(fixtureText("callers/caller1.ts")).ast;
  const imported = tsParser.parseForESLint(fixtureText("shared/jobs.ts")).ast;
  const owned = { ast: imported };
  const load = vi.fn(() => owned);
  const store = { load, resolve: () => "resolved.ts" };
  const first = fixtureFile("callers/caller0.ts");
  const next = fixtureFile("callers/caller1.ts");
  // Oxlint resets this same Context/SourceCode between files.
  const context = { filename: first, sourceCode: { ast: firstAst } };
  const file = currentSources(context, store);
  assert.equal(file.load(first).ast, firstAst);
  assert.equal(file.load(first), file.load(first));
  assert.equal(file.load(first).context, context);
  assert.equal(load.mock.calls.length, 0);
  assert.equal(file.resolve(), "resolved.ts");
  context.filename = next;
  context.sourceCode.ast = nextAst;
  const following = currentSources(context, store);
  assert.equal(following.load(next).ast, nextAst);
  assert.equal(following.load(first), owned);
  assert.notEqual(following.load(first).ast, firstAst);
});

it("reuses owned imported records across per-file facades in the CLI owner", () => {
  const probe = instrumentSources();
  try {
    withArgv(cli, [], () => {
      const context = { filename: fixtureFile("callers/caller0.ts") };
      const records = [];
      const facades = [];
      const hooks = withSourceOwner({
        create() {
          const sources = createImportedSources(context);
          facades.push(sources);
          records.push(sources.load(sources.resolve("@session/jobs", context.filename)));
          return { Program() {} };
        },
      }).createOnce(context);
      for (let file = 0; file < 2; file++) {
        hooks.before();
        hooks.after();
      }
      assert.ok(records[0] === records[1]);
      assert.notEqual(facades[0], facades[1]);
    });
    assert.equal(probe.stats.parses, 1);
    assert.deepEqual([...probe.stats.reads.values()], [1, 1]);
  } finally {
    probe.restore();
  }
});

it("memoizes an incompatible optional resolver for the CLI owner", () => {
  const load = Module._load;
  let attempts = 0;
  const peer = vi.spyOn(Module, "_load").mockImplementation(function (name, ...args) {
    const value = load.call(this, name, ...args);
    if (name !== "typescript") return value;
    attempts++;
    return { ...value, createModuleResolutionCache: undefined };
  });
  try {
    withArgv(cli, [], () => {
      const context = {};
      const hooks = withSourceOwner({
        create() {
          assert.equal(createImportedSources(context), null);
          return { Program() {} };
        },
      }).createOnce(context);
      for (let file = 0; file < 2; file++) {
        hooks.before();
        hooks.after();
      }
      assert.equal(attempts, 1);
    });
  } finally {
    peer.mockRestore();
  }
});

it("memoizes unavailable tooling only for the one-shot CLI owner", () => {
  const load = Module._load;
  let attempts = 0;
  const peer = vi.spyOn(Module, "_load").mockImplementation(function (name, ...args) {
    if (name === "typescript") {
      attempts++;
      throw new Error("Peer unavailable");
    }
    return load.call(this, name, ...args);
  });
  try {
    withArgv(cli, [], () => {
      const context = {};
      const hooks = withSourceOwner({
        create() {
          assert.equal(createImportedSources(context), null);
          assert.equal(sourceOwner(context).sources, false);
          return { Program() {} };
        },
      }).createOnce(context);
      for (let file = 0; file < 2; file++) {
        hooks.before();
        hooks.after();
      }
      assert.equal(attempts, 1);
    });
    assert.equal(createImportedSources({}), null);
    assert.equal(attempts, 2);
  } finally {
    peer.mockRestore();
  }
});

import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, isAbsolute, resolve } from "node:path";
import { Linter } from "eslint";
import tsParser from "@typescript-eslint/parser";
import { expect, it } from "vitest";
import { __dirname, plugin } from "./helpers.mjs";

const root = resolve(
  __dirname,
  "../../../test-cases/eslint-plugin/async-call-disposition/fixture/proven-consumers",
);
const targets = [{ sourceSpecifierPatterns: ["@app/jobs"], calleeNamePatterns: ["enqueueEmail"] }];
const propertyConsumer = {
  sourceSpecifierPatterns: ["@app/consumer"],
  calleeNamePatterns: ["consume"],
  callbackArgumentProperties: [
    { argumentIndex: 0, propertyNames: ["enqueueEmail", "enqueueEmailCallback"] },
  ],
};

// File snapshots keep fixture-relative paths and a fixed order. Absolute paths
// and diagnostic order would change across machines and worker counts.
const slashRoot = root.replaceAll("\\", "/").replace(/\/$/, "");
const tempRoots = [tmpdir(), process.env.TEMP, process.env.TMP, process.env.TMPDIR]
  .filter(Boolean)
  .map((item) => item.replaceAll("\\", "/").replace(/\/$/, ""))
  .sort((left, right) => right.length - left.length);

function normalizeText(text) {
  let value = String(text).replaceAll("\\", "/");
  if (slashRoot) value = value.replaceAll(slashRoot, "<root>");
  for (const prefix of tempRoots) value = value.replaceAll(prefix, "<temp>");
  return value
    .replace(
      /(?<![A-Za-z0-9_>])(?:[A-Za-z]:)?(?:\/private\/var\/folders|\/var\/folders)\/[^/\s]+\/[^/\s]+\/T(?:\/\S*)?/gi,
      "<temp>",
    )
    .replace(
      /(?<![A-Za-z0-9_>])(?:[A-Za-z]:)?\/Users\/[^/\s]+\/AppData\/Local\/Temp(?:\/\S*)?/gi,
      "<temp>",
    )
    .replace(/(?<![A-Za-z0-9_>])(?:\/private\/tmp|\/var\/tmp|\/tmp)(?:\/\S*)?/gi, "<temp>")
    .replace(/\b(pid|ppid)\s*[:=]\s*\d+\b/gi, "$1 <pid>")
    .replace(/\b\d+(?:\.\d+)?\s*(?:ms|us|µs|ns)\b/gi, "<timing>");
}

function relativeFile(filename) {
  const absolute = isAbsolute(filename) ? filename : resolve(process.cwd(), filename);
  const slash = normalizeText(absolute);
  return slash.startsWith("<root>/") ? slash.slice("<root>/".length) : slash;
}

function compareDiagnostics(left, right) {
  return (
    left.file.localeCompare(right.file) ||
    left.line - right.line ||
    left.column - right.column ||
    String(left.messageId || left.code).localeCompare(String(right.messageId || right.code))
  );
}

function sortValue(value) {
  if (Array.isArray(value)) return value.map(sortValue);
  if (value && typeof value === "object") {
    return Object.fromEntries(
      Object.keys(value)
        .sort()
        .map((key) => [key, sortValue(value[key])]),
    );
  }
  return value;
}

function verify(name, typed = false, awaitingConsumers, parser = tsParser) {
  const filename = resolve(root, name);
  return {
    filename,
    messages: new Linter({ cwd: dirname(filename) }).verify(
      readFileSync(filename, "utf8"),
      {
        files: ["**/*.ts"],
        languageOptions: {
          parser,
          parserOptions: typed
            ? { project: resolve(root, "tsconfig.json"), disallowAutomaticSingleRunInference: true }
            : {},
        },
        plugins: { "no-mistakes": plugin },
        rules: { "no-mistakes/async-call-disposition": ["error", { targets, awaitingConsumers }] },
      },
      { filename },
    ),
  };
}

function reports(name, typed = false, awaitingConsumers, parser = tsParser) {
  return verify(name, typed, awaitingConsumers, parser).messages.map(({ line, messageId }) => [
    line,
    messageId,
  ]);
}

function eslintDiagnostics(file, mode) {
  const typed = mode === "typed";
  const awaitingConsumers = mode === "properties" ? [propertyConsumer] : undefined;
  const { filename, messages } = verify(file, typed, awaitingConsumers);
  return messages
    .map((item) => ({
      column: item.column,
      file: relativeFile(item.filename || filename),
      line: item.line,
      message: normalizeText(item.message),
      messageId: item.messageId,
    }))
    .sort(compareDiagnostics);
}

function oxlintDiagnostics(configName, files, typeAware) {
  const args = [
    resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
    "--config",
    resolve(root, configName),
    "--format",
    "json",
  ];
  if (typeAware) args.splice(1, 0, "--type-aware", "--deny-warnings");
  args.push(...files.map((file) => resolve(root, file)));
  const result = spawnSync(process.execPath, args, { encoding: "utf8" });
  const payload = JSON.parse(result.stdout || "null");
  assert.ok(payload && Array.isArray(payload.diagnostics), result.stderr);
  return {
    diagnostics: payload.diagnostics
      .map((item) => ({
        code: item.code,
        column: item.labels?.[0]?.span?.column ?? null,
        file: relativeFile(item.filename),
        line: item.labels?.[0]?.span?.line ?? null,
        message: normalizeText(item.message ?? ""),
      }))
      .sort(compareDiagnostics),
    files: [...files].sort(),
    status: result.status,
    typeAware,
  };
}

const eslintCases = [
  ["alias/caller.ts", "untyped"],
  ["cross-file-chain.ts", "typed"],
  ["cross-file-chain.ts", "untyped"],
  ["cross-file-forwarding.ts", "untyped"],
  ["cross-file-hostile.ts", "typed"],
  ["cross-file-hostile.ts", "untyped"],
  ["cross-file-prototypes.ts", "typed"],
  ["cross-file-prototypes.ts", "untyped"],
  ["cross-file-receivers.ts", "untyped"],
  ["cross-file.ts", "typed"],
  ["cross-file.ts", "untyped"],
  ["imported.ts", "typed"],
  ["imported.ts", "untyped"],
  ["invalid-config/caller.ts", "untyped"],
  ["invalid-options/caller.ts", "untyped"],
  ["local.ts", "typed"],
  ["local.ts", "untyped"],
  ["missing-extends/caller.ts", "untyped"],
  ["no-resolve/caller.ts", "untyped"],
  ["properties.ts", "properties"],
  ["properties.ts", "untyped"],
].sort((left, right) => left[0].localeCompare(right[0]) || left[1].localeCompare(right[1]));

it("snapshots proven-consumer fixture diagnostics", async () => {
  const document = sortValue({
    eslint: eslintCases.map(([file, mode]) => ({
      diagnostics: eslintDiagnostics(file, mode),
      file,
      mode,
    })),
    oxlint: [
      oxlintDiagnostics(".oxlintrc.json", ["imported.ts"], false),
      oxlintDiagnostics(".oxlintrc.json", ["local.ts"], false),
      oxlintDiagnostics(".oxlintrc.json", ["properties.ts"], false),
      oxlintDiagnostics(".oxlintrc-proven.json", ["cross-file-chain.ts", "cross-file.ts"], true),
    ],
  });
  await expect(`${JSON.stringify(document, null, 2)}\n`).toMatchFileSnapshot(
    "snapshots/async-call-disposition-proven-consumers.snap",
  );
});

it("does not enumerate the parser program outside the resolved implementation", () => {
  const parser = {
    ...tsParser,
    parseForESLint(...args) {
      const parsed = tsParser.parseForESLint(...args);
      parsed.services.program = new Proxy(parsed.services.program, {
        get(target, key) {
          assert.notEqual(key, "getSourceFiles", "consumer proofs must stay scoped");
          const value = Reflect.get(target, key);
          return typeof value === "function" ? value.bind(target) : value;
        },
      });
      return parsed;
    },
  };
  assert.deepEqual(reports("imported.ts", true, undefined, parser), reports("imported.ts", true));
});

it("validates exact dependency property contracts", () => {
  for (const callbackArgumentProperties of [
    [],
    [{ argumentIndex: -1, propertyNames: ["enqueueEmail"] }],
    [{ argumentIndex: 0, propertyNames: [] }],
    [{ argumentIndex: 0, propertyNames: ["enqueueEmail", "enqueueEmail"] }],
    [{ argumentIndex: 0, propertyNames: [""] }],
  ]) {
    assert.throws(() =>
      reports("properties.ts", false, [{ ...propertyConsumer, callbackArgumentProperties }]),
    );
  }
});

it.each(["noResolve", "missingMap"])(
  "preserves parser-owned %s refusal without syntactic fallback",
  (mode) => {
    const parser = {
      ...tsParser,
      parseForESLint(...args) {
        const parsed = tsParser.parseForESLint(...args);
        if (mode === "missingMap") parsed.services.esTreeNodeToTSNodeMap = undefined;
        else {
          const original = parsed.services.program;
          parsed.services.program = new Proxy(original, {
            get(target, key) {
              if (key === "getCompilerOptions")
                return () => ({ ...target.getCompilerOptions(), noResolve: true });
              const value = Reflect.get(target, key);
              return typeof value === "function" ? value.bind(target) : value;
            },
          });
        }
        return parsed;
      },
    };
    assert.deepEqual(
      reports("imported.ts", true, undefined, parser),
      [3, 5, 7, 9, 11, 13, 17, 19, 21, 23, 25, 28, 30, 32].map((line) => [line, "disposition"]),
    );
  },
);

import { execFile, spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, isAbsolute, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { promisify } from "node:util";
import tsParser from "@typescript-eslint/parser";
import { Linter } from "eslint";

const execFileAsync = promisify(execFile);

export const require = createRequire(import.meta.url);
export const __dirname = dirname(fileURLToPath(import.meta.url));
export const plugin = require("../src");
export const oxlintBin = resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint");

const lintersByCwd = new Map();

export function getLinter(cwd = process.cwd()) {
  let linter = lintersByCwd.get(cwd);
  if (!linter) {
    linter = new Linter({ configType: "flat", cwd });
    lintersByCwd.set(cwd, linter);
  }
  return linter;
}

function buildOxlintArgs({ config, files, typeAware = false, denyWarnings = false }) {
  return [
    oxlintBin,
    ...(typeAware ? ["--type-aware"] : []),
    ...(denyWarnings ? ["--deny-warnings"] : []),
    "--config",
    config,
    "--format",
    "json",
    ...files,
  ];
}

function formatOxlintResult({ status, stdout, stderr, error, signal }, cwd) {
  const base = cwd || process.cwd();
  const diagnostics = stdout ? JSON.parse(stdout).diagnostics || [] : [];
  const byFile = new Map();
  for (const item of diagnostics) {
    const key = resolve(base, item.filename || ".");
    const list = byFile.get(key);
    if (list) list.push(item);
    else byFile.set(key, [item]);
  }
  return {
    status,
    stdout,
    stderr,
    error,
    signal,
    diagnostics,
    forFile(filePath, ruleId) {
      const items = byFile.get(resolve(base, filePath)) || [];
      return ruleId ? items.filter((item) => item.code?.includes(ruleId)) : items;
    },
  };
}

export function runOxlint({
  config,
  files,
  cwd,
  typeAware = false,
  denyWarnings = false,
  timeout,
  maxBuffer = 8 * 1024 * 1024,
  env,
}) {
  const result = spawnSync(
    process.execPath,
    buildOxlintArgs({ config, files, typeAware, denyWarnings }),
    {
      cwd,
      encoding: "utf8",
      timeout,
      maxBuffer,
      ...(env ? { env } : {}),
    },
  );
  return formatOxlintResult(result, cwd);
}

export async function runOxlintAsync({
  config,
  files,
  cwd,
  typeAware = false,
  denyWarnings = false,
  timeout,
  maxBuffer = 8 * 1024 * 1024,
  env,
}) {
  try {
    const { stdout, stderr } = await execFileAsync(
      process.execPath,
      buildOxlintArgs({ config, files, typeAware, denyWarnings }),
      {
        cwd,
        encoding: "utf8",
        timeout,
        maxBuffer,
        ...(env ? { env } : {}),
      },
    );
    return formatOxlintResult({ status: 0, stdout, stderr, error: undefined, signal: null }, cwd);
  } catch (error) {
    if (typeof error?.code === "number") {
      return formatOxlintResult(
        {
          status: error.code,
          stdout: error.stdout || "",
          stderr: error.stderr || "",
          error: undefined,
          signal: error.signal ?? null,
        },
        cwd,
      );
    }
    throw error;
  }
}

export function fixture(name) {
  return readFileSync(
    resolve(__dirname, "../../../test-cases/eslint-snippets/fixture", name),
    "utf8",
  );
}

// Flat config matches paths inside the linter cwd. Fixture files live outside
// this package, so those anchor cwd at the file. An absolute path inside the
// package keeps the package cwd: path-scoped rules match that relative path
// (`backend/index.ts`), and the file directory would collapse it to the basename.
export function linterCwd(filename) {
  if (!isAbsolute(filename)) return process.cwd();
  const fromPackage = relative(process.cwd(), filename);
  if (fromPackage === "" || fromPackage.startsWith("..") || isAbsolute(fromPackage)) {
    return dirname(filename);
  }
  return process.cwd();
}

export function lint(code, rules, filename = "fixture.jsx", globals = {}, sourceType = "module") {
  const linter = getLinter(linterCwd(filename));
  const isTypeScript = /\.[cm]?tsx?$/.test(filename);
  return linter.verify(
    code,
    {
      files: ["**/*.{js,jsx,ts,tsx,mjs,mts,cjs,cts}"],
      languageOptions: {
        ecmaVersion: 2024,
        sourceType,
        globals,
        ...(isTypeScript ? { parser: tsParser } : {}),
        parserOptions: { ecmaFeatures: { jsx: true } },
      },
      plugins: {
        "no-mistakes": plugin,
      },
      rules,
    },
    { filename },
  );
}

export function messages(
  code,
  rule,
  option,
  filename = "fixture.jsx",
  globals = {},
  sourceType = "module",
) {
  const config =
    option === undefined
      ? { [`no-mistakes/${rule}`]: "error" }
      : { [`no-mistakes/${rule}`]: ["error", option] };
  return lint(code, config, filename, globals, sourceType).map((message) => message.messageId);
}

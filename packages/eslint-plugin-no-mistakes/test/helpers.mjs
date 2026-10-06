import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, isAbsolute, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import tsParser from "@typescript-eslint/parser";
import { Linter } from "eslint";

export const require = createRequire(import.meta.url);
export const __dirname = dirname(fileURLToPath(import.meta.url));
export const plugin = require("../src");

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
function linterCwd(filename) {
  if (!isAbsolute(filename)) return process.cwd();
  const fromPackage = relative(process.cwd(), filename);
  if (fromPackage === "" || fromPackage.startsWith("..") || isAbsolute(fromPackage)) {
    return dirname(filename);
  }
  return process.cwd();
}

export function lint(code, rules, filename = "fixture.jsx", globals = {}) {
  const linter = new Linter({
    configType: "flat",
    cwd: linterCwd(filename),
  });
  const isTypeScript = /\.[cm]?tsx?$/.test(filename);
  return linter.verify(
    code,
    {
      files: ["**/*.{js,jsx,ts,tsx,mjs,mts,cjs,cts}"],
      languageOptions: {
        ecmaVersion: 2024,
        sourceType: "module",
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

export function messages(code, rule, option, filename = "fixture.jsx", globals = {}) {
  const config =
    option === undefined
      ? { [`no-mistakes/${rule}`]: "error" }
      : { [`no-mistakes/${rule}`]: ["error", option] };
  return lint(code, config, filename, globals).map((message) => message.messageId);
}

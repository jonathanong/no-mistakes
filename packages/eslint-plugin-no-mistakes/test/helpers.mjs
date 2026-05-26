import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, resolve } from "node:path";
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

export function lint(code, rules, filename = "fixture.jsx", globals = {}) {
  const linter = new Linter({ configType: "flat" });
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

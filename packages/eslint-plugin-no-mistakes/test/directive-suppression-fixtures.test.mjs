import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import tsParser from "@typescript-eslint/parser";
import { expect, it } from "vitest";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { __dirname } from "./helpers.mjs";

const languageOptions = {
  ecmaVersion: 2024,
  sourceType: "module",
  globals: {},
  parserOptions: { ecmaFeatures: { jsx: true } },
};

function source(rule, name) {
  return readFileSync(
    resolve(__dirname, "../../../test-cases/eslint-snippets/fixture", rule, name),
    "utf8",
  );
}

const groups = [
  {
    rule: "no-delete-property",
    ext: "mjs",
    forms: ["line", "next-line", "file"],
    filename: (form) => `samples/${form}.mjs`,
  },
  {
    rule: "no-global-fetch-outside-helper",
    ext: "mjs",
    forms: ["line", "next-line", "file"],
    filename: (form) => `web/app/${form}.mjs`,
    options: [{ checkedPathPatterns: ["web/**"] }],
  },
  {
    rule: "no-inline-noop-promise-catch",
    ext: "mjs",
    forms: ["line", "next-line", "file"],
    filename: (form) => `${form}.mjs`,
  },
  {
    rule: "no-placeholder-never-type-exports",
    ext: "ts",
    forms: ["line", "next-line", "file"],
    filename: (form) => `${form}.ts`,
    typescript: true,
  },
  {
    rule: "no-vitest-sequential",
    ext: "test.mjs",
    forms: ["file"],
    filename: (form) => `${form}.test.mjs`,
  },
  {
    rule: "nextjs-metadata-exports-location",
    ext: "mjs",
    forms: ["line", "next-line", "file"],
    filename: (form) => `app/components/${form}.mjs`,
  },
  {
    rule: "nextjs-no-manual-script-tags",
    ext: "jsx",
    forms: ["line", "next-line", "file"],
    filename: (form) => `app/${form}.jsx`,
  },
  {
    rule: "nextjs-static-fetch-method",
    ext: "mjs",
    forms: ["line", "next-line", "file"],
    filename: (form) => `${form}.mjs`,
  },
  {
    rule: "nextjs-static-fetch-url",
    ext: "mjs",
    forms: ["line", "next-line", "file"],
    filename: (form) => `${form}.mjs`,
  },
  {
    rule: "playwright-assertion-timeout-cap",
    ext: "spec.mjs",
    forms: ["line", "next-line", "file"],
    filename: (form) => `e2e/${form}.spec.mjs`,
  },
  {
    rule: "playwright-consistent-attribute",
    ext: "jsx",
    forms: ["line", "next-line", "file"],
    filename: (form) => `${form}.jsx`,
  },
  {
    rule: "playwright-defaults",
    ext: "jsx",
    forms: ["line", "next-line", "file"],
    filename: (form) => `${form}.jsx`,
  },
  {
    rule: "playwright-literals",
    ext: "jsx",
    forms: ["line", "file"],
    filename: (form) => `${form}.jsx`,
  },
  {
    rule: "playwright-naming-convention",
    ext: "jsx",
    forms: ["line", "next-line", "file"],
    filename: (form) => `${form}.jsx`,
  },
  {
    rule: "playwright-no-empty",
    ext: "jsx",
    forms: ["line", "next-line", "file"],
    filename: (form) => `${form}.jsx`,
  },
  {
    rule: "playwright-no-hoisted-unique-token",
    ext: "spec.mjs",
    forms: ["line", "next-line", "file"],
    filename: (form) => `e2e/${form}.spec.mjs`,
    options: [{ tokenFactories: ["randomSuffix"] }],
  },
  {
    rule: "playwright-no-raw-scroll-pagination",
    ext: "spec.mjs",
    forms: ["line", "file"],
    filename: (form) => `e2e/${form}.spec.mjs`,
  },
  {
    rule: "playwright-prefer-get-by-test-id",
    ext: "mjs",
    forms: ["line", "file"],
    filename: (form) => `${form}.mjs`,
  },
  {
    rule: "playwright-require-exported-component-attribute",
    ext: "jsx",
    forms: ["line", "next-line", "file"],
    filename: (form) => `${form}.jsx`,
  },
  {
    rule: "playwright-require-interactive-test-id",
    ext: "jsx",
    forms: ["line", "file"],
    filename: (form) => `${form}.jsx`,
  },
  {
    rule: "playwright-selector-priority",
    ext: "spec.mjs",
    forms: ["line", "file"],
    filename: (form) => `e2e/${form}.spec.mjs`,
  },
  {
    rule: "playwright-test-timeout-cap",
    ext: "spec.mjs",
    forms: ["line"],
    filename: (form) => `e2e/${form}.spec.mjs`,
  },
  {
    rule: "playwright-unique",
    ext: "jsx",
    forms: ["line", "next-line", "file"],
    filename: (form) => `${form}.jsx`,
  },
];

const document = {};
for (const group of groups) {
  const valid = [];
  const invalid = [];
  for (const form of ["control", ...group.forms]) {
    const file = `suppression-${form}.${group.ext}`;
    const item = { code: source(group.rule, file), filename: group.filename(form) };
    if (group.options) item.options = group.options;
    if (group.typescript) item.languageOptions = { parser: tsParser };
    (form === "control" ? invalid : valid).push(item);
  }
  document[group.rule] = snapshotSuite(group.rule, languageOptions, { valid, invalid });
}

it("snapshots missing no-mistakes suppression directives", async () => {
  await expect(snapshotDocument(document)).toMatchFileSnapshot(
    "snapshots/directive-suppression-fixtures.snap",
  );
});

import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { describe, it } from "vitest";

import { __dirname } from "./helpers.mjs";

describe("oxlint support", () => {
  it("loads the plugin through jsPlugins", () => {
    const root = mkdtempSync(join(tmpdir(), "pac-oxlint-"));
    try {
      writeFileSync(join(root, "fixture.jsx"), "<button data-pw={id} />;\n");
      writeFileSync(
        join(root, ".oxlintrc.json"),
        JSON.stringify({
          jsPlugins: [{ name: "no-mistakes", specifier: resolve(__dirname, "../src/index.js") }],
          rules: { "no-mistakes/playwright-literals": "error" },
        }),
      );
      const result = spawnSync(
        process.execPath,
        [
          resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
          "--config",
          ".oxlintrc.json",
          "fixture.jsx",
        ],
        {
          cwd: root,
          encoding: "utf8",
        },
      );
      assert.notEqual(result.status, 0);
      assert.match(`${result.stderr || ""}${result.stdout || ""}`, /literal|test ID/i);
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  });

  it("reports const aliases through the plugin", () => {
    const root = mkdtempSync(join(tmpdir(), "pac-oxlint-"));
    try {
      writeFileSync(join(root, "fixture.ts"), "const alias = original;\n");
      writeFileSync(
        join(root, ".oxlintrc.json"),
        JSON.stringify({
          jsPlugins: [{ name: "no-mistakes", specifier: resolve(__dirname, "../src/index.js") }],
          rules: { "no-mistakes/ts-no-const-aliases": "error" },
        }),
      );
      const result = spawnSync(
        process.execPath,
        [
          resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
          "--config",
          ".oxlintrc.json",
          "fixture.ts",
        ],
        {
          cwd: root,
          encoding: "utf8",
        },
      );
      assert.notEqual(result.status, 0);
      assert.match(`${result.stderr || ""}${result.stdout || ""}`, /const alias|indirection/i);
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  });

  it("reports inline no-op Promise catches through the plugin", () => {
    const root = mkdtempSync(join(tmpdir(), "pac-oxlint-"));
    try {
      writeFileSync(join(root, "fixture.ts"), "Promise.resolve().catch(() => {});\n");
      writeFileSync(
        join(root, ".oxlintrc.json"),
        JSON.stringify({
          jsPlugins: [{ name: "no-mistakes", specifier: resolve(__dirname, "../src/index.js") }],
          rules: { "no-mistakes/no-inline-noop-promise-catch": "error" },
        }),
      );
      const result = spawnSync(
        process.execPath,
        [
          resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
          "--config",
          ".oxlintrc.json",
          "fixture.ts",
        ],
        {
          cwd: root,
          encoding: "utf8",
        },
      );
      assert.notEqual(result.status, 0);
      assert.match(
        `${result.stderr || ""}${result.stdout || ""}`,
        /no-op Promise catch|noopCatch/i,
      );
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  });

  it("reports delayed rejection observers through the plugin", () => {
    const root = mkdtempSync(join(tmpdir(), "pac-oxlint-"));
    try {
      writeFileSync(
        join(root, "fixture.ts"),
        `import { expect } from "vitest";

async function test() {
  const operation = startOperation();
  await release();
  await expect(operation).rejects.toThrow();
}
`,
      );
      writeFileSync(
        join(root, ".oxlintrc.json"),
        JSON.stringify({
          jsPlugins: [{ name: "no-mistakes", specifier: resolve(__dirname, "../src/index.js") }],
          rules: { "no-mistakes/test-no-delayed-rejects": "error" },
        }),
      );
      const result = spawnSync(
        process.execPath,
        [
          resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
          "--config",
          ".oxlintrc.json",
          "fixture.ts",
        ],
        {
          cwd: root,
          encoding: "utf8",
        },
      );
      assert.notEqual(result.status, 0);
      assert.match(`${result.stderr || ""}${result.stdout || ""}`, /rejection|observer|await/i);
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  });

  it("reports aliased imported calls missing required options", () => {
    const root = mkdtempSync(join(tmpdir(), "pac-oxlint-"));
    try {
      writeFileSync(
        join(root, "fixture.ts"),
        'import { validateUrl as checkUrl } from "ssrf-guard/node";\ncheckUrl(url);\n',
      );
      writeFileSync(
        join(root, ".oxlintrc.json"),
        JSON.stringify({
          jsPlugins: [{ name: "no-mistakes", specifier: resolve(__dirname, "../src/index.js") }],
          rules: {
            "no-mistakes/require-options-on-imported-call": [
              "error",
              {
                targets: [
                  {
                    sourceSpecifierPatterns: ["ssrf-guard/node"],
                    calleeNamePatterns: ["validateUrl"],
                    optionsPosition: 2,
                    requiredProperties: ["timeoutMs", "signal"],
                  },
                ],
              },
            ],
          },
        }),
      );
      const result = spawnSync(
        process.execPath,
        [
          resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
          "--config",
          ".oxlintrc.json",
          "fixture.ts",
        ],
        {
          cwd: root,
          encoding: "utf8",
        },
      );
      assert.notEqual(result.status, 0);
      assert.match(
        `${result.stderr || ""}${result.stdout || ""}`,
        /statically visible options object|require-options-on-imported-call/i,
      );
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  });
});

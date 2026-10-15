import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { Linter } from "eslint";
import tsParser from "@typescript-eslint/parser";
import { describe, it } from "vitest";
import { fixture, plugin, __dirname } from "./helpers.mjs";

describe("vitest-timeout-cap program state", () => {
  it("never reads preceding programs when the host reuses its rule context", () => {
    let current;
    const shared = new Proxy(
      {},
      {
        get(_target, key) {
          const value = Reflect.get(current, key, current);
          return typeof value === "function" ? value.bind(current) : value;
        },
      },
    );
    const reads = [];
    const original = plugin.rules["vitest-timeout-cap"];
    const wrapped = {
      ...original,
      create(context) {
        current = context;
        const index = reads.push(0) - 1;
        const visitors = original.create(shared);
        return {
          ...visitors,
          CallExpression(node) {
            const args = node.arguments;
            // Count actual resolver scans of retained calls, without timing assertions.
            Object.defineProperty(node, "arguments", {
              configurable: true,
              enumerable: true,
              get() {
                reads[index]++;
                return args;
              },
            });
            visitors.CallExpression(node);
          },
        };
      },
    };
    const linter = new Linter({ configType: "flat" });
    const config = {
      files: ["**/*.ts"],
      languageOptions: { parser: tsParser },
      plugins: { proof: { rules: { timeout: wrapped } } },
      rules: {
        "proof/timeout": [
          "error",
          {
            defaultMax: 30000,
            overrideMax: 30000,
            unknownValues: "finding",
          },
        ],
      },
    };
    for (let index = 0; index < 8; index++) {
      const previous = reads.slice();
      assert.deepEqual(
        linter.verify(fixture("vitest-timeout-cap/program-state-ordinary.ts"), config, {
          filename: `ordinary-${index}.ts`,
        }),
        [],
      );
      assert.deepEqual(reads.slice(0, index), previous);
      assert.ok(reads[index] > 0);
    }
    const previous = reads.slice();
    const messages = linter.verify(
      fixture("vitest-timeout-cap/program-state-controls.ts"),
      config,
      { filename: "controls.test.ts" },
    );
    assert.deepEqual(
      messages.map(({ line, messageId }) => [line, messageId]),
      [
        [3, "timeout"],
        [6, "timeout"],
        [9, "unknown"],
      ],
    );
    assert.deepEqual(reads.slice(0, previous.length), previous);
  });

  it("preserves positive, callback-alias and unknown findings in public Oxlint", () => {
    const root = resolve(
      __dirname,
      "../../../test-cases/eslint-snippets/fixture/vitest-timeout-cap",
    );
    const result = spawnSync(
      process.execPath,
      [
        resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
        "--config",
        resolve(root, "program-state-oxlint.json"),
        "--disable-nested-config",
        "--no-ignore",
        "--format=json",
        resolve(root, "program-state-ordinary.ts"),
        resolve(root, "program-state-controls.ts"),
      ],
      { encoding: "utf8" },
    );
    assert.equal(result.status, 1, result.stderr + result.stdout);
    const report = JSON.parse(result.stdout);
    assert.equal(report.diagnostics.length, 3, result.stdout);
    assert.ok(
      report.diagnostics.every(({ filename }) => filename.endsWith("program-state-controls.ts")),
    );
  });
});

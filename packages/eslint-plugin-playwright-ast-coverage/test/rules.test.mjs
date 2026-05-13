import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { Linter } from "eslint";
import { describe, it } from "vitest";

const require = createRequire(import.meta.url);
const __dirname = dirname(fileURLToPath(import.meta.url));
const plugin = require("../src");

function fixture(name) {
  return readFileSync(resolve(__dirname, "fixtures", name), "utf8");
}

function lint(code, rules, filename = "fixture.jsx") {
  const linter = new Linter({ configType: "flat" });
  return linter.verify(code, {
    files: ["**/*.{js,jsx}"],
    languageOptions: {
      ecmaVersion: 2024,
      sourceType: "module",
      parserOptions: { ecmaFeatures: { jsx: true } },
    },
    plugins: {
      "playwright-ast-coverage": plugin,
    },
    rules,
  }, { filename });
}

function messages(code, rule, option) {
  const config = option === undefined
    ? { [`playwright-ast-coverage/${rule}`]: "error" }
    : { [`playwright-ast-coverage/${rule}`]: ["error", option] };
  return lint(code, config).map((message) => message.messageId);
}

describe("plugin exports", () => {
  it("exposes rules and flat configs", () => {
    assert.equal(plugin.meta.name, "eslint-plugin-playwright-ast-coverage");
    assert.ok(plugin.rules.literals);
    assert.equal(plugin.configs.recommended.rules["playwright-ast-coverage/literals"], "error");
    assert.deepEqual(plugin.configs.strict.rules["playwright-ast-coverage/consistent-attribute"], [
      "error",
      { canonicalAttribute: "data-pw" },
    ]);
  });
});

describe("literals", () => {
  it("accepts literals, expression literals, empty static templates, static templates, and defaulted props", () => {
    assert.deepEqual(messages(fixture("literals-valid.jsx"), "literals", { allowStaticTemplates: true }), []);
  });

  it("reports missing, dynamic, non-defaulted, and forbidden template values", () => {
    assert.deepEqual(messages(fixture("literals-invalid.jsx"), "literals"), ["literal", "literal", "literal", "literal", "literal", "literal"]);
  });

  it("rejects templates without static text and allows defaulted identifiers outside props", () => {
    const code = `
      function A(testId = "save") {
        helper();
        page.getByTestId(testId);
        return <button data-pw={\`\${id}\`} />;
      }
    `;
    assert.deepEqual(messages(code, "literals", { allowStaticTemplates: true }), ["literal"]);
  });

  it("can be configured for literal-only mode and custom attributes", () => {
    const code = `
      const A = ({ testId = "save" }) => <button data-qa={testId} />;
    `;
    assert.deepEqual(messages(code, "literals", {
      selectorAttributes: ["data-qa"],
      allowDefaultedProps: false,
    }), ["literal"]);
  });
});

describe("defaults", () => {
  it("requires literal defaults for prop passthrough", () => {
    assert.deepEqual(messages(fixture("defaults.jsx"), "defaults"), ["default", "default"]);
  });
});

describe("unique", () => {
  it("reports duplicate exact literals within a file", () => {
    assert.deepEqual(messages(fixture("unique.jsx"), "unique"), ["duplicate", "duplicate"]);
    assert.deepEqual(messages("<button data-pw={id} />;", "unique"), []);
  });
});

describe("no-empty", () => {
  it("reports empty literal test IDs", () => {
    assert.deepEqual(messages("<><button data-pw='' /><button data-pw /><button data-pw={'ok'} /></>;", "no-empty"), ["empty"]);
  });
});

describe("consistent-attribute", () => {
  it("requires the configured canonical attribute", () => {
    assert.deepEqual(messages("<button data-testid='save' />;", "consistent-attribute"), ["attribute"]);
    assert.deepEqual(messages("<button data-pw='save' />;", "consistent-attribute"), []);
    assert.deepEqual(messages("<button data-qa='save' />;", "consistent-attribute", {
      selectorAttributes: ["data-qa"],
      canonicalAttribute: "data-qa",
    }), []);
  });
});

describe("require-interactive-test-id", () => {
  it("reports interactive elements without a test ID", () => {
    assert.equal(messages(fixture("interactive.jsx"), "require-interactive-test-id").length, 15);
  });
});

describe("prefer-get-by-test-id", () => {
  it("reports exact CSS test-id selectors in Playwright selector calls", () => {
    assert.deepEqual(messages(fixture("prefer-get-by-testid.js"), "prefer-get-by-test-id"), ["prefer", "prefer", "prefer", "prefer", "prefer", "prefer", "prefer"]);
  });
});

describe("naming-convention", () => {
  it("checks literal values against a configurable pattern", () => {
    assert.deepEqual(messages("<><button data-pw='SaveButton' /><button data-pw='save-button' /></>;", "naming-convention"), ["naming"]);
    assert.deepEqual(messages("<button data-pw='SaveButton' />;", "naming-convention", { pattern: "^[A-Z][A-Za-z]+$" }), []);
  });
});

describe("strict config", () => {
  it("runs the strict rule set", () => {
    const messages = lint("<button data-testid='Save' />;", plugin.configs.strict.rules);
    assert.deepEqual(messages.map((message) => message.ruleId).sort(), [
      "playwright-ast-coverage/consistent-attribute",
      "playwright-ast-coverage/naming-convention",
    ]);
  });
});

describe("oxlint support", () => {
  it("loads the plugin through jsPlugins", () => {
    const root = mkdtempSync(join(tmpdir(), "pac-oxlint-"));
    try {
      writeFileSync(join(root, "fixture.jsx"), "<button data-pw={id} />;\n");
      writeFileSync(join(root, ".oxlintrc.json"), JSON.stringify({
        jsPlugins: [{ name: "playwright-ast-coverage", specifier: resolve(__dirname, "../src/index.js") }],
        rules: { "playwright-ast-coverage/literals": "error" },
      }));
      const result = spawnSync(resolve(__dirname, "../../../node_modules/.bin/oxlint"), ["--config", ".oxlintrc.json", "fixture.jsx"], {
        cwd: root,
        encoding: "utf8",
      });
      assert.notEqual(result.status, 0);
      assert.match(`${result.stderr || ""}${result.stdout || ""}`, /literal|test ID/i);
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  });
});

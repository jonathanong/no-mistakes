import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, it } from "vitest";
import { __dirname, plugin } from "./helpers.mjs";

describe("docs coverage", () => {
  it("documents every exported ESLint rule", () => {
    const repo = resolve(__dirname, "../../..");
    const index = readFileSync(resolve(repo, "docs/eslint-rules/README.md"), "utf8");
    for (const ruleId of Object.keys(plugin.rules).sort()) {
      const file = `${ruleId}.md`;
      const path = resolve(repo, "docs/eslint-rules", file);
      assert.ok(existsSync(path), `missing docs for ${ruleId}`);
      assert.ok(index.includes(file), `docs/eslint-rules/README.md must link ${file}`);
      const body = readFileSync(path, "utf8");
      assert.match(body, /Why:/, `${file} needs a Why section`);
      assert.match(body, /Counterexample:/, `${file} needs a counterexample`);
      assert.match(body, /Fix:/, `${file} needs fix guidance`);
    }
  });

  it("documents presets and public rule options", () => {
    const repo = resolve(__dirname, "../../..");
    const index = readFileSync(resolve(repo, "docs/eslint-rules/README.md"), "utf8");
    const pluginDoc = readFileSync(resolve(repo, "docs/eslint-plugin.md"), "utf8");

    for (const preset of Object.keys(plugin.configs).sort()) {
      assert.ok(index.includes(`configs.${preset}`), `missing configs.${preset} in rule index`);
      assert.ok(pluginDoc.includes(`configs.${preset}`), `missing configs.${preset} in plugin doc`);
    }

    for (const optionName of [
      "selectorAttributes",
      "interactiveComponents",
      "canonicalAttribute",
      "allowInlineScriptIds",
      "allowInlineScriptIdPatterns",
      "includePathPatterns",
      "allowDefaultReExports",
      "allowDefaultedProps",
      "allowStaticTemplates",
      "allowBeforeAllAssignments",
      "targets",
      "handlers",
      "sourceSpecifierPatterns",
      "calleeNamePatterns",
    ]) {
      assert.ok(pluginDoc.includes(optionName), `missing option ${optionName}`);
    }
  });
});

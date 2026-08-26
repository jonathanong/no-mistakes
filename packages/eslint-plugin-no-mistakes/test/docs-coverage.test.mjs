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
      for (const section of [
        "Why",
        "Disallowed",
        "Allowed",
        "Options",
        "Fix",
        "Suppression",
        "Related rules",
      ]) {
        assert.match(body, new RegExp(`^## ${section}$`, "m"), `${file} needs ${section}`);
      }
      assert.ok(
        (body.match(/```/g) ?? []).length >= 6,
        `${file} needs invalid, valid, and suppression examples`,
      );

      const optionNames = schemaPropertyNames(plugin.rules[ruleId].meta.schema);
      for (const optionName of optionNames) {
        assert.ok(body.includes(`\`${optionName}\``), `${file} must document ${optionName}`);
      }
      if (optionNames.length === 0) {
        assert.match(
          body,
          /This rule has no options\.|The schema accepts an options object\./,
          `${file} must state whether it has options`,
        );
      }
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

    for (const [ruleId, rule] of Object.entries(plugin.rules)) {
      const optionNames = schemaPropertyNames(rule.meta.schema);
      if (optionNames.length === 0) continue;
      assert.ok(pluginDoc.includes(`\`${ruleId}\``), `missing ${ruleId} option entry`);
      for (const optionName of optionNames) {
        assert.ok(
          pluginDoc.includes(`\`${optionName}`),
          `missing ${optionName} in central option reference`,
        );
      }
    }

    assert.match(pluginDoc, /Only two rules expose editor suggestions\./);
    assert.match(pluginDoc, /`async-call-disposition` offers a\n`void` prefix/);
    assert.match(pluginDoc, /`async-try-catch-return-await`\noffers `await`/);
  });
});

function schemaPropertyNames(schema) {
  const names = new Set();
  const visit = (value) => {
    if (!value || typeof value !== "object") return;
    for (const [name, child] of Object.entries(value.properties ?? {})) {
      names.add(name);
      visit(child);
    }
    if (Array.isArray(value.items)) value.items.forEach(visit);
    else visit(value.items);
  };
  schema.forEach(visit);
  return [...names].sort();
}

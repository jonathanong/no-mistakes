import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { describe, expect, it } from "vitest";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { lint, plugin } from "./helpers.mjs";

const require = createRequire(import.meta.url);
const { isFetchCall } = require("../src/helpers");
const rule = "nextjs-static-fetch-url";
const languageOptions = {
  ecmaVersion: 2024,
  sourceType: "module",
  globals: {},
  parserOptions: { ecmaFeatures: { jsx: true } },
};

function urlCase(code, filename = "fixture.jsx", globals) {
  const item = { code, filename };
  if (globals) item.languageOptions = { globals };
  return item;
}

const suite = {
  valid: [
    urlCase("fetch('https://api.example.com/users');"),
    urlCase('fetch("https://api.example.com/users");'),
    urlCase("fetch(`https://api.example.com/users`);"),
    urlCase("fetch('https://api.example.com/users', { cache: 'force-cache' });"),
    urlCase("function f(fetch) { fetch(url); }"),
    urlCase("const fetch = mockFetch; fetch(url);"),
    urlCase("request('https://api.example.com/users');"),
    urlCase("client.fetch('https://api.example.com/users');"),
    urlCase("import { fetch } from 'undici'; fetch(url);"),
    urlCase("class fetch {} fetch(url);"),
  ],
  invalid: [
    // 1 dynamic.
    urlCase("fetch(url);"),
    urlCase("fetch(`https://api.example.com/${id}`);"),
    urlCase("fetch(getUrl());"),
    urlCase("fetch(base + path);"),
    urlCase("fetch();"),
    // 1 dynamic. A configured fetch global is not a shadow.
    urlCase("fetch(url);", "fixture.js", { fetch: "readonly" }),
  ],
};

describe("plugin exports", () => {
  it("exposes rules and flat configs", () => {
    assert.equal(plugin.meta.name, "eslint-plugin-no-mistakes");
    assert.ok(plugin.rules["nextjs-static-fetch-url"]);
    assert.ok(plugin.rules["nextjs-static-fetch-method"]);
    assert.equal(plugin.configs.recommended.rules["no-mistakes/nextjs-static-fetch-url"], "error");
    assert.equal(
      plugin.configs.recommended.rules["no-mistakes/nextjs-static-fetch-method"],
      "error",
    );
  });
});

describe("nextjs-static-fetch-url", () => {
  it("snapshots nextjs-rules diagnostics", async () => {
    await expect(
      snapshotDocument({
        [rule]: snapshotSuite(rule, languageOptions, suite),
      }),
    ).toMatchFileSnapshot("snapshots/nextjs-rules.snap");
  }, 30_000);

  it("supports fallback shadow checks when scope.set.get is not a function", () => {
    const fakeScope = {
      set: { get: 123 },
      variables: [{ name: "fetch", defs: [{ type: "Variable" }] }],
      upper: null,
    };
    const context = {
      sourceCode: {
        getScope: () => fakeScope,
      },
    };
    assert.equal(
      isFetchCall(
        { type: "CallExpression", callee: { type: "Identifier", name: "fetch" } },
        context,
      ),
      false,
    );
  });
});

describe("recommended config", () => {
  it("runs the recommended rule set", () => {
    const results = lint("fetch(url);", plugin.configs.recommended.rules);
    const ruleIds = results.map((m) => m.ruleId).sort();
    assert.deepEqual(ruleIds, ["no-mistakes/nextjs-static-fetch-url"]);
  });
});

import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { isFetchCall } from "../src/helpers.js";
import { lint, messages, plugin } from "./helpers.mjs";

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
  it("accepts string literal URLs", () => {
    assert.deepEqual(
      messages("fetch('https://api.example.com/users');", "nextjs-static-fetch-url"),
      [],
    );
    assert.deepEqual(
      messages('fetch("https://api.example.com/users");', "nextjs-static-fetch-url"),
      [],
    );
  });

  it("accepts expression-free template literals", () => {
    assert.deepEqual(
      messages("fetch(`https://api.example.com/users`);", "nextjs-static-fetch-url"),
      [],
    );
  });

  it("accepts fetch with static URL and options", () => {
    assert.deepEqual(
      messages(
        "fetch('https://api.example.com/users', { cache: 'force-cache' });",
        "nextjs-static-fetch-url",
      ),
      [],
    );
  });

  it("reports identifier URLs", () => {
    assert.deepEqual(messages("fetch(url);", "nextjs-static-fetch-url"), ["dynamic"]);
  });

  it("reports template literal URLs with expressions", () => {
    assert.deepEqual(
      messages("fetch(`https://api.example.com/${id}`);", "nextjs-static-fetch-url"),
      ["dynamic"],
    );
  });

  it("reports call expression URLs", () => {
    assert.deepEqual(messages("fetch(getUrl());", "nextjs-static-fetch-url"), ["dynamic"]);
  });

  it("reports binary expression URLs", () => {
    assert.deepEqual(messages("fetch(base + path);", "nextjs-static-fetch-url"), ["dynamic"]);
  });

  it("reports missing URL argument", () => {
    assert.deepEqual(messages("fetch();", "nextjs-static-fetch-url"), ["dynamic"]);
  });

  it("does not report when fetch is shadowed by a parameter", () => {
    assert.deepEqual(messages("function f(fetch) { fetch(url); }", "nextjs-static-fetch-url"), []);
  });

  it("does not report when fetch is shadowed by a local variable", () => {
    assert.deepEqual(
      messages("const fetch = mockFetch; fetch(url);", "nextjs-static-fetch-url"),
      [],
    );
  });

  it("does not report on non-fetch call expressions", () => {
    assert.deepEqual(
      messages("request('https://api.example.com/users');", "nextjs-static-fetch-url"),
      [],
    );
  });

  it("does not report on method calls named fetch", () => {
    assert.deepEqual(
      messages("client.fetch('https://api.example.com/users');", "nextjs-static-fetch-url"),
      [],
    );
  });

  it("does not treat fetch configured as a global as shadowed", () => {
    assert.deepEqual(
      messages("fetch(url);", "nextjs-static-fetch-url", undefined, "fixture.js", {
        fetch: "readonly",
      }),
      ["dynamic"],
    );
  });

  it("does not report when fetch is shadowed by an import", () => {
    assert.deepEqual(
      messages("import { fetch } from 'undici'; fetch(url);", "nextjs-static-fetch-url"),
      [],
    );
  });

  it("does not report when fetch is shadowed by a class", () => {
    assert.deepEqual(messages("class fetch {} fetch(url);", "nextjs-static-fetch-url"), []);
  });

  it("supports fallback shadow checks when scope.set is unavailable", () => {
    const fakeScope = {
      set: undefined,
      variables: [{ name: "fetch", defs: [{ type: "Variable" }] }],
      upper: null,
    };
    const context = {
      sourceCode: {
        getScope: () => fakeScope,
      },
    };
    assert.equal(isFetchCall({ callee: { type: "Identifier", name: "fetch" } }, context), false);
  });
});

describe("recommended config", () => {
  it("runs the recommended rule set", () => {
    const results = lint("fetch(url);", plugin.configs.recommended.rules);
    const ruleIds = results.map((m) => m.ruleId).sort();
    assert.deepEqual(ruleIds, ["no-mistakes/nextjs-static-fetch-url"]);
  });
});

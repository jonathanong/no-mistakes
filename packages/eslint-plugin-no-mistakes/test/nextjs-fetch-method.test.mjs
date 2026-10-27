import { describe, expect, it } from "vitest";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";

const rule = "nextjs-static-fetch-method";
const languageOptions = {
  ecmaVersion: 2024,
  sourceType: "module",
  globals: {},
  parserOptions: { ecmaFeatures: { jsx: true } },
};

function methodCase(code) {
  return { code, filename: "fixture.jsx" };
}

const suite = {
  valid: [
    methodCase("fetch('https://api.example.com');"),
    methodCase("fetch('https://api.example.com', {});"),
    methodCase("fetch('https://api.example.com', { method: 'POST' });"),
    methodCase('fetch("https://api.example.com", { method: "GET" });'),
    methodCase("fetch('https://api.example.com', { method: `POST` });"),
    methodCase("fetch('https://api.example.com', { cache: 'no-store' });"),
    methodCase("fetch('https://api.example.com', opts);"),
    methodCase("fetch('https://api.example.com', { ...opts });"),
    methodCase("fetch('https://api.example.com', { ['method']: 'GET' });"),
    methodCase("fetch('https://api.example.com', { 'method': 'POST' });"),
    // The last duplicate method property is the literal.
    methodCase("fetch('https://api.example.com', { method: verb, method: 'GET' });"),
    methodCase("function f(fetch) { fetch('url', { method: verb }); }"),
  ],
  invalid: [
    // 1 dynamic. Computed method key with a non-literal value.
    methodCase("fetch('https://api.example.com', { ['method']: verb });"),
    methodCase("fetch('https://api.example.com', { [`method`]: verb });"),
    // 1 dynamic. String literal method key with a non-literal value.
    methodCase("fetch('https://api.example.com', { 'method': verb });"),
    // 1 dynamic.
    methodCase("fetch('https://api.example.com', { method: method });"),
    // 1 dynamic. The last duplicate method property wins.
    methodCase("fetch('https://api.example.com', { method: 'GET', method: verb });"),
    // 1 dynamic.
    methodCase("fetch('https://api.example.com', { method: getMethod() });"),
    // 1 dynamic.
    methodCase("fetch('https://api.example.com', { method: `${verb}` });"),
  ],
};

describe("nextjs-static-fetch-method", () => {
  it("snapshots nextjs-fetch-method diagnostics", async () => {
    await expect(
      snapshotDocument({
        [rule]: snapshotSuite(rule, languageOptions, suite),
      }),
    ).toMatchFileSnapshot("snapshots/nextjs-fetch-method.snap");
  }, 30_000);
});

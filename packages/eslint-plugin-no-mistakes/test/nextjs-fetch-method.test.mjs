import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { messages } from "./helpers.mjs";

describe("nextjs-static-fetch-method", () => {
  it("accepts fetch without options", () => {
    assert.deepEqual(
      messages("fetch('https://api.example.com');", "nextjs-static-fetch-method"),
      [],
    );
  });

  it("accepts fetch with empty options", () => {
    assert.deepEqual(
      messages("fetch('https://api.example.com', {});", "nextjs-static-fetch-method"),
      [],
    );
  });

  it("accepts fetch with string literal method", () => {
    assert.deepEqual(
      messages(
        "fetch('https://api.example.com', { method: 'POST' });",
        "nextjs-static-fetch-method",
      ),
      [],
    );
    assert.deepEqual(
      messages(
        'fetch("https://api.example.com", { method: "GET" });',
        "nextjs-static-fetch-method",
      ),
      [],
    );
  });

  it("accepts fetch with expression-free template method", () => {
    assert.deepEqual(
      messages(
        "fetch('https://api.example.com', { method: `POST` });",
        "nextjs-static-fetch-method",
      ),
      [],
    );
  });

  it("accepts fetch with no method property", () => {
    assert.deepEqual(
      messages(
        "fetch('https://api.example.com', { cache: 'no-store' });",
        "nextjs-static-fetch-method",
      ),
      [],
    );
  });

  it("accepts fetch with non-object second argument", () => {
    assert.deepEqual(
      messages("fetch('https://api.example.com', opts);", "nextjs-static-fetch-method"),
      [],
    );
  });

  it("accepts fetch with spread-only options", () => {
    assert.deepEqual(
      messages("fetch('https://api.example.com', { ...opts });", "nextjs-static-fetch-method"),
      [],
    );
  });

  it("accepts fetch with computed method key", () => {
    assert.deepEqual(
      messages(
        "fetch('https://api.example.com', { ['method']: 'GET' });",
        "nextjs-static-fetch-method",
      ),
      [],
    );
  });

  it("reports computed method key with non-literal value", () => {
    assert.deepEqual(
      messages(
        "fetch('https://api.example.com', { ['method']: verb });",
        "nextjs-static-fetch-method",
      ),
      ["dynamic"],
    );
    assert.deepEqual(
      messages(
        "fetch('https://api.example.com', { [`method`]: verb });",
        "nextjs-static-fetch-method",
      ),
      ["dynamic"],
    );
  });

  it("accepts fetch with string literal method key and literal value", () => {
    assert.deepEqual(
      messages(
        "fetch('https://api.example.com', { 'method': 'POST' });",
        "nextjs-static-fetch-method",
      ),
      [],
    );
  });

  it("reports string literal method key with non-literal value", () => {
    assert.deepEqual(
      messages(
        "fetch('https://api.example.com', { 'method': verb });",
        "nextjs-static-fetch-method",
      ),
      ["dynamic"],
    );
  });

  it("reports identifier method values", () => {
    assert.deepEqual(
      messages(
        "fetch('https://api.example.com', { method: method });",
        "nextjs-static-fetch-method",
      ),
      ["dynamic"],
    );
  });

  it("uses the last duplicate method property", () => {
    assert.deepEqual(
      messages(
        "fetch('https://api.example.com', { method: 'GET', method: verb });",
        "nextjs-static-fetch-method",
      ),
      ["dynamic"],
    );
    assert.deepEqual(
      messages(
        "fetch('https://api.example.com', { method: verb, method: 'GET' });",
        "nextjs-static-fetch-method",
      ),
      [],
    );
  });

  it("reports call expression method values", () => {
    assert.deepEqual(
      messages(
        "fetch('https://api.example.com', { method: getMethod() });",
        "nextjs-static-fetch-method",
      ),
      ["dynamic"],
    );
  });

  it("reports template literal method with expressions", () => {
    assert.deepEqual(
      messages(
        "fetch('https://api.example.com', { method: `${verb}` });",
        "nextjs-static-fetch-method",
      ),
      ["dynamic"],
    );
  });

  it("does not report when fetch is shadowed", () => {
    assert.deepEqual(
      messages(
        "function f(fetch) { fetch('url', { method: verb }); }",
        "nextjs-static-fetch-method",
      ),
      [],
    );
  });
});

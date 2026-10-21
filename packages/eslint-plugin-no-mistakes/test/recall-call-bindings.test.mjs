import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { fixture, messages } from "./helpers.mjs";

const jobs = {
  targets: [
    {
      sourceSpecifierPatterns: ["@app/jobs"],
      calleeNamePatterns: ["enqueueEmail"],
    },
  ],
};

const handlers = {
  handlers: [
    {
      sourceSpecifierPatterns: ["@app/errors"],
      calleeNamePatterns: ["report"],
    },
  ],
};

const ssrf = {
  targets: [
    {
      sourceSpecifierPatterns: ["ssrf-guard/node"],
      calleeNamePatterns: ["validateUrl"],
      optionsPosition: 2,
      requiredProperties: ["timeoutMs", "signal"],
    },
  ],
};

const banned = {
  checkedPathPatterns: ["src/**"],
  bannedImports: [{ module: "@acme/rate-limit", names: ["invalidateAll"] }],
};

const server = {
  getterCalleePatterns: ["^serverApi\\.get$"],
  requiredWrapperCallee: "nullableEntity",
};

describe("call-binding recall", () => {
  it("follows default imports, static keys, and forwarded calls", () => {
    assert.equal(
      messages(
        fixture("async-call-disposition/recall.mjs"),
        "async-call-disposition",
        jobs,
        "recall.mjs",
      ).length,
      6,
    );
    assert.equal(
      messages(
        fixture("async-call-disposition/call-method.mjs"),
        "async-call-disposition",
        {
          targets: [
            {
              sourceSpecifierPatterns: ["@app/jobs"],
              calleeNamePatterns: ["call", "enqueueEmail"],
            },
          ],
        },
        "call-method.mjs",
      ).length,
      2,
    );
    assert.equal(
      messages(
        fixture("async-try-catch-return-await/recall.mjs"),
        "async-try-catch-return-await",
        handlers,
        "recall.mjs",
      ).length,
      2,
    );
    assert.deepEqual(
      messages(
        fixture("gh-api-argument-safety/recall.mjs"),
        "gh-api-argument-safety",
        undefined,
        "recall.mjs",
      ),
      Array(7).fill("endpointQuery"),
    );
    assert.equal(
      messages(
        fixture("require-options-on-imported-call/recall.mjs"),
        "require-options-on-imported-call",
        ssrf,
        "recall.mjs",
      ).length,
      6,
    );
    assert.equal(
      messages(
        fixture("no-argv-main-guard/recall.mjs"),
        "no-argv-main-guard",
        undefined,
        "recall.mjs",
      ).length,
      7,
    );
    assert.equal(
      messages(
        fixture("no-inline-noop-promise-catch/recall.mjs"),
        "no-inline-noop-promise-catch",
        undefined,
        "recall.mjs",
      ).length,
      4,
    );
    assert.equal(
      messages(
        fixture("await-array-methods/recall.mjs"),
        "await-array-methods",
        undefined,
        "recall.mjs",
      ).length,
      6,
    );
    assert.equal(
      messages(
        fixture("nextjs-static-fetch-method/recall.mjs"),
        "nextjs-static-fetch-method",
        undefined,
        "recall.mjs",
      ).length,
      6,
    );
    assert.equal(
      messages(
        fixture("nextjs-static-fetch-url/recall.mjs"),
        "nextjs-static-fetch-url",
        undefined,
        "recall.mjs",
      ).length,
      3,
    );
    assert.equal(
      messages(
        fixture("module-mock-boundary/recall.mjs"),
        "module-mock-boundary",
        undefined,
        "recall.mjs",
      ).length,
      6,
    );
    assert.equal(
      messages(
        fixture("module-mock-preserve-exports/recall.mjs"),
        "module-mock-preserve-exports",
        undefined,
        "recall.mjs",
      ).length,
      3,
    );
    assert.equal(
      messages(
        fixture("no-global-fetch-outside-helper/recall.mjs"),
        "no-global-fetch-outside-helper",
        { checkedPathPatterns: ["**"] },
        "recall.mjs",
      ).length,
      5,
    );
    assert.equal(
      messages(
        fixture("no-banned-import-outside-allowed-paths/recall.mjs"),
        "no-banned-import-outside-allowed-paths",
        banned,
        "src/app/recall.mjs",
      ).length,
      2,
    );
    assert.equal(
      messages(
        fixture("server-require-nullable-fetch-wrapper/recall.ts"),
        "server-require-nullable-fetch-wrapper",
        server,
        "backend/users.ts",
      ).length,
      6,
    );
  });

  it("keeps a configured local wrapper and a forwarded allowed origin", () => {
    const local = fixture("server-require-nullable-fetch-wrapper/local-alias.ts");
    assert.deepEqual(
      messages(
        local,
        "server-require-nullable-fetch-wrapper",
        {
          ...server,
          requiredWrapperCallee: "wrap",
        },
        "backend/users.ts",
      ),
      [],
    );
    assert.deepEqual(
      messages(local, "server-require-nullable-fetch-wrapper", server, "backend/users.ts"),
      [],
    );
    assert.deepEqual(
      messages(
        fixture("no-inline-noop-promise-catch/forwarded-origin.mjs"),
        "no-inline-noop-promise-catch",
        { allowedCalleeNamePatterns: ["releaseLock"] },
        "forwarded-origin.mjs",
      ),
      ["noopCatch"],
    );
  });
});

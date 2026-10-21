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
    assert.deepEqual(
      messages(
        fixture("async-call-disposition/comma-alias.mjs"),
        "async-call-disposition",
        jobs,
        "comma-alias.mjs",
      ),
      ["disposition", "disposition", "disposition"],
    );
    assert.deepEqual(
      messages(
        fixture("async-call-disposition/instantiated-alias.ts"),
        "async-call-disposition",
        jobs,
        "instantiated-alias.ts",
      ),
      ["disposition", "disposition"],
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
        fixture("require-options-on-imported-call/apply-method.mjs"),
        "require-options-on-imported-call",
        {
          targets: [
            {
              receiverPatterns: ["client"],
              calleeNamePatterns: ["apply"],
              optionsPosition: 2,
              requiredProperties: ["signal"],
            },
          ],
        },
        "apply-method.mjs",
      ).length,
      1,
    );
    assert.equal(
      messages(
        fixture("module-mock-boundary/dynamic-key.mjs"),
        "module-mock-boundary",
        undefined,
        "dynamic-key.mjs",
      ).length,
      1,
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
        fixture("no-inline-noop-promise-catch/asserted.ts"),
        "no-inline-noop-promise-catch",
        undefined,
        "asserted.ts",
      ).length,
      2,
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
    assert.deepEqual(
      messages(
        fixture("await-array-methods/asserted-alias.ts"),
        "await-array-methods",
        undefined,
        "asserted-alias.ts",
      ),
      ["awaited", "awaited"],
    );
    assert.deepEqual(
      messages(
        fixture("await-array-methods/shadowed-alias.mjs"),
        "await-array-methods",
        undefined,
        "shadowed-alias.mjs",
      ),
      ["awaited", "awaited"],
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
        fixture("nextjs-static-fetch-url/dynamic-fetch-key.mjs"),
        "nextjs-static-fetch-url",
        undefined,
        "dynamic-fetch-key.mjs",
      ).length,
      1,
    );
    assert.equal(
      messages(
        fixture("nextjs-static-fetch-url/asserted-alias.ts"),
        "nextjs-static-fetch-url",
        undefined,
        "asserted-alias.ts",
      ).length,
      1,
    );
    assert.deepEqual(
      messages(
        fixture("nextjs-static-fetch-url/asserted-global.ts"),
        "nextjs-static-fetch-url",
        undefined,
        "asserted-global.ts",
      ),
      ["dynamic", "dynamic"],
    );
    assert.deepEqual(
      messages(
        fixture("nextjs-static-fetch-url/asserted-global.ts"),
        "nextjs-static-fetch-method",
        undefined,
        "asserted-global.ts",
      ),
      ["dynamic", "dynamic"],
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
        fixture("module-mock-preserve-exports/asserted.ts"),
        "module-mock-preserve-exports",
        undefined,
        "asserted.ts",
      ).length,
      2,
    );
    assert.deepEqual(
      messages(
        fixture("module-mock-preserve-exports/asserted-method.ts"),
        "module-mock-preserve-exports",
        undefined,
        "asserted-method.ts",
      ),
      ["preserve", "preserve"],
    );
    assert.deepEqual(
      messages(
        fixture("module-mock-preserve-exports/aliased-spread.mjs"),
        "module-mock-preserve-exports",
        undefined,
        "aliased-spread.mjs",
      ),
      ["preserve", "preserve"],
    );
    assert.deepEqual(
      messages(
        fixture("module-mock-preserve-exports/aliased-spread.mjs"),
        "module-mock-boundary",
        undefined,
        "aliased-spread.mjs",
      ),
      ["boundary", "boundary"],
    );
    assert.deepEqual(
      messages(
        fixture("module-mock-preserve-exports/dynamic-method-key.mjs"),
        "module-mock-preserve-exports",
        undefined,
        "dynamic-method-key.mjs",
      ),
      ["preserve", "preserve", "preserve"],
    );
    assert.deepEqual(
      messages(
        fixture("module-mock-preserve-exports/dynamic-method-key.mjs"),
        "module-mock-boundary",
        undefined,
        "dynamic-method-key.mjs",
      ),
      ["boundary", "boundary", "boundary"],
    );
    assert.deepEqual(
      messages(
        fixture("module-mock-preserve-exports/factory-less-alias.mjs"),
        "module-mock-preserve-exports",
        undefined,
        "factory-less-alias.mjs",
      ),
      ["preserve"],
    );
    assert.deepEqual(
      messages(
        fixture("module-mock-preserve-exports/factory-less-alias.mjs"),
        "module-mock-boundary",
        undefined,
        "factory-less-alias.mjs",
      ),
      ["boundary", "boundary", "boundary"],
    );
    assert.deepEqual(
      messages(
        fixture("module-mock-preserve-exports/reassigned-alias.mjs"),
        "module-mock-preserve-exports",
        undefined,
        "reassigned-alias.mjs",
      ),
      ["preserve", "preserve"],
    );
    assert.deepEqual(
      messages(
        fixture("module-mock-preserve-exports/reassigned-alias.mjs"),
        "module-mock-boundary",
        undefined,
        "reassigned-alias.mjs",
      ),
      ["boundary", "boundary"],
    );
    assert.deepEqual(
      messages(
        fixture("module-mock-preserve-exports/comma-alias.mjs"),
        "module-mock-preserve-exports",
        undefined,
        "comma-alias.mjs",
      ),
      ["preserve", "preserve"],
    );
    assert.deepEqual(
      messages(
        fixture("module-mock-preserve-exports/comma-alias.mjs"),
        "module-mock-boundary",
        undefined,
        "comma-alias.mjs",
      ),
      ["boundary", "boundary"],
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
        fixture("no-global-fetch-outside-helper/dynamic-fetch-key.mjs"),
        "no-global-fetch-outside-helper",
        { checkedPathPatterns: ["**"] },
        "dynamic-fetch-key.mjs",
      ).length,
      2,
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
    assert.deepEqual(
      messages(
        fixture("server-require-nullable-fetch-wrapper/forwarded-this.ts"),
        "server-require-nullable-fetch-wrapper",
        server,
        "backend/users.ts",
      ),
      ["wrapper", "wrapper"],
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
    const getterAlias = messages(
      fixture("server-require-nullable-fetch-wrapper/local-getter-alias.ts"),
      "server-require-nullable-fetch-wrapper",
      {
        getterCalleePatterns: ["^getUser$"],
        requiredWrapperCallee: "wrap",
      },
      "backend/users.ts",
    );
    assert.deepEqual(getterAlias, ["wrapper"]);
    assert.equal(
      messages(
        fixture("server-require-nullable-fetch-wrapper/named-call.ts"),
        "server-require-nullable-fetch-wrapper",
        {
          getterCalleePatterns: ["^rpc\\.call$"],
          requiredWrapperCallee: "nullableEntity",
        },
        "backend/users.ts",
      ).length,
      2,
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
    assert.deepEqual(
      messages(
        fixture("no-inline-noop-promise-catch/allowed-call-origin.mjs"),
        "no-inline-noop-promise-catch",
        { allowedCalleeNamePatterns: ["call", "apply"] },
        "allowed-call-origin.mjs",
      ),
      ["noopCatch"],
    );
    assert.equal(
      messages(
        fixture("no-inline-noop-promise-catch/allowed-call-origin.mjs"),
        "no-inline-noop-promise-catch",
        { allowedCalleeNamePatterns: ["fetch"] },
        "allowed-call-origin.mjs",
      ).length,
      3,
    );
  });
});

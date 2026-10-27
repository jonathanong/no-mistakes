import tsParser from "@typescript-eslint/parser";
import { describe, expect, it } from "vitest";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { fixture } from "./helpers.mjs";

const languageOptions = {
  ecmaVersion: 2024,
  sourceType: "module",
  globals: {},
  parserOptions: { ecmaFeatures: { jsx: true } },
};
const typescript = { parser: tsParser, sourceType: "module", globals: {} };

function lintCase(code, filename, option) {
  const item = { code, filename };
  if (/\.[cm]?tsx?$/.test(filename)) item.languageOptions = typescript;
  if (option !== undefined) item.options = [option];
  return item;
}

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
  it("snapshots remaining diagnostics", async () => {
    const localGetter = fixture("server-require-nullable-fetch-wrapper/local-getter-name.ts");
    const localAlias = fixture("server-require-nullable-fetch-wrapper/local-alias.ts");
    const applyOptions = {
      targets: [
        {
          receiverPatterns: ["client"],
          calleeNamePatterns: ["apply"],
          optionsPosition: 2,
          requiredProperties: ["signal"],
        },
      ],
    };
    await expect(
      snapshotDocument({
        "async-call-disposition": snapshotSuite("async-call-disposition", languageOptions, {
          invalid: [
            // Six findings through a default import and forwarded calls.
            lintCase(fixture("async-call-disposition/recall.mjs"), "recall.mjs", jobs),
            // Static call-method and enqueueEmail selectors both remain recognized.
            lintCase(fixture("async-call-disposition/call-method.mjs"), "call-method.mjs", {
              targets: [
                {
                  sourceSpecifierPatterns: ["@app/jobs"],
                  calleeNamePatterns: ["call", "enqueueEmail"],
                },
              ],
            }),
            lintCase(fixture("async-call-disposition/comma-alias.mjs"), "comma-alias.mjs", jobs),
            lintCase(
              fixture("async-call-disposition/instantiated-alias.ts"),
              "instantiated-alias.ts",
              jobs,
            ),
            lintCase(
              fixture("async-call-disposition/namespace-alias.mjs"),
              "namespace-alias.mjs",
              jobs,
            ),
          ],
        }),
        "async-try-catch-return-await": snapshotSuite(
          "async-try-catch-return-await",
          languageOptions,
          {
            invalid: [
              // 2 diagnostics.
              lintCase(fixture("async-try-catch-return-await/recall.mjs"), "recall.mjs", handlers),
            ],
          },
        ),
        "await-array-methods": snapshotSuite("await-array-methods", languageOptions, {
          invalid: [
            // 6 diagnostics.
            lintCase(fixture("await-array-methods/recall.mjs"), "recall.mjs"),
            lintCase(fixture("await-array-methods/asserted-alias.ts"), "asserted-alias.ts"),
            lintCase(fixture("await-array-methods/shadowed-alias.mjs"), "shadowed-alias.mjs"),
          ],
        }),
        "gh-api-argument-safety": snapshotSuite("gh-api-argument-safety", languageOptions, {
          invalid: [
            // 7 endpointQuery diagnostics.
            lintCase(fixture("gh-api-argument-safety/recall.mjs"), "recall.mjs"),
          ],
        }),
        "module-mock-boundary": snapshotSuite("module-mock-boundary", languageOptions, {
          invalid: [
            // 1 diagnostic.
            lintCase(fixture("module-mock-boundary/dynamic-key.mjs"), "dynamic-key.mjs"),
            // 6 diagnostics.
            lintCase(fixture("module-mock-boundary/recall.mjs"), "recall.mjs"),
            lintCase(
              fixture("module-mock-preserve-exports/aliased-spread.mjs"),
              "aliased-spread.mjs",
            ),
            lintCase(
              fixture("module-mock-preserve-exports/dynamic-method-key.mjs"),
              "dynamic-method-key.mjs",
            ),
            lintCase(
              fixture("module-mock-preserve-exports/factory-less-alias.mjs"),
              "factory-less-alias.mjs",
            ),
            lintCase(
              fixture("module-mock-preserve-exports/reassigned-alias.mjs"),
              "reassigned-alias.mjs",
            ),
            lintCase(fixture("module-mock-preserve-exports/comma-alias.mjs"), "comma-alias.mjs"),
            lintCase(
              fixture("module-mock-preserve-exports/asserted-alias.ts"),
              "asserted-alias.ts",
            ),
          ],
        }),
        "module-mock-preserve-exports": snapshotSuite(
          "module-mock-preserve-exports",
          languageOptions,
          {
            invalid: [
              // 3 diagnostics.
              lintCase(fixture("module-mock-preserve-exports/recall.mjs"), "recall.mjs"),
              // 2 diagnostics.
              lintCase(fixture("module-mock-preserve-exports/asserted.ts"), "asserted.ts"),
              lintCase(
                fixture("module-mock-preserve-exports/asserted-method.ts"),
                "asserted-method.ts",
              ),
              lintCase(
                fixture("module-mock-preserve-exports/aliased-spread.mjs"),
                "aliased-spread.mjs",
              ),
              lintCase(
                fixture("module-mock-preserve-exports/dynamic-method-key.mjs"),
                "dynamic-method-key.mjs",
              ),
              lintCase(
                fixture("module-mock-preserve-exports/factory-less-alias.mjs"),
                "factory-less-alias.mjs",
              ),
              lintCase(
                fixture("module-mock-preserve-exports/reassigned-alias.mjs"),
                "reassigned-alias.mjs",
              ),
              lintCase(fixture("module-mock-preserve-exports/comma-alias.mjs"), "comma-alias.mjs"),
              lintCase(
                fixture("module-mock-preserve-exports/asserted-alias.ts"),
                "asserted-alias.ts",
              ),
            ],
          },
        ),
        "nextjs-static-fetch-method": snapshotSuite("nextjs-static-fetch-method", languageOptions, {
          invalid: [
            // 6 diagnostics.
            lintCase(fixture("nextjs-static-fetch-method/recall.mjs"), "recall.mjs"),
            lintCase(fixture("nextjs-static-fetch-url/asserted-global.ts"), "asserted-global.ts"),
          ],
        }),
        "nextjs-static-fetch-url": snapshotSuite("nextjs-static-fetch-url", languageOptions, {
          invalid: [
            // 3 diagnostics.
            lintCase(fixture("nextjs-static-fetch-url/recall.mjs"), "recall.mjs"),
            // 1 diagnostic.
            lintCase(
              fixture("nextjs-static-fetch-url/dynamic-fetch-key.mjs"),
              "dynamic-fetch-key.mjs",
            ),
            // 1 diagnostic.
            lintCase(fixture("nextjs-static-fetch-url/asserted-alias.ts"), "asserted-alias.ts"),
            lintCase(fixture("nextjs-static-fetch-url/asserted-global.ts"), "asserted-global.ts"),
          ],
        }),
        "no-argv-main-guard": snapshotSuite("no-argv-main-guard", languageOptions, {
          invalid: [
            // 7 diagnostics.
            lintCase(fixture("no-argv-main-guard/recall.mjs"), "recall.mjs"),
          ],
        }),
        "no-banned-import-outside-allowed-paths": snapshotSuite(
          "no-banned-import-outside-allowed-paths",
          languageOptions,
          {
            invalid: [
              // 2 diagnostics.
              lintCase(
                fixture("no-banned-import-outside-allowed-paths/recall.mjs"),
                "src/app/recall.mjs",
                banned,
              ),
            ],
          },
        ),
        "no-global-fetch-outside-helper": snapshotSuite(
          "no-global-fetch-outside-helper",
          languageOptions,
          {
            invalid: [
              // 5 diagnostics.
              lintCase(fixture("no-global-fetch-outside-helper/recall.mjs"), "recall.mjs", {
                checkedPathPatterns: ["**"],
              }),
              // 2 diagnostics.
              lintCase(
                fixture("no-global-fetch-outside-helper/dynamic-fetch-key.mjs"),
                "dynamic-fetch-key.mjs",
                { checkedPathPatterns: ["**"] },
              ),
            ],
          },
        ),
        "no-inline-noop-promise-catch": snapshotSuite(
          "no-inline-noop-promise-catch",
          languageOptions,
          {
            invalid: [
              // 4 diagnostics.
              lintCase(fixture("no-inline-noop-promise-catch/recall.mjs"), "recall.mjs"),
              // 2 diagnostics.
              lintCase(fixture("no-inline-noop-promise-catch/asserted.ts"), "asserted.ts"),
              lintCase(
                fixture("no-inline-noop-promise-catch/forwarded-origin.mjs"),
                "forwarded-origin.mjs",
                { allowedCalleeNamePatterns: ["releaseLock"] },
              ),
              lintCase(
                fixture("no-inline-noop-promise-catch/allowed-call-origin.mjs"),
                "allowed-call-origin.mjs",
                { allowedCalleeNamePatterns: ["call", "apply"] },
              ),
              // 3 diagnostics.
              lintCase(
                fixture("no-inline-noop-promise-catch/allowed-call-origin.mjs"),
                "allowed-call-origin.mjs",
                { allowedCalleeNamePatterns: ["fetch"] },
              ),
            ],
          },
        ),
        "require-options-on-imported-call": snapshotSuite(
          "require-options-on-imported-call",
          languageOptions,
          {
            invalid: [
              // 6 diagnostics.
              lintCase(fixture("require-options-on-imported-call/recall.mjs"), "recall.mjs", ssrf),
              // 1 diagnostic.
              lintCase(
                fixture("require-options-on-imported-call/apply-method.mjs"),
                "apply-method.mjs",
                applyOptions,
              ),
            ],
          },
        ),
        "server-require-nullable-fetch-wrapper": snapshotSuite(
          "server-require-nullable-fetch-wrapper",
          languageOptions,
          {
            valid: [
              // A configured local wrapper and a forwarded allowed origin stay silent.
              lintCase(localAlias, "backend/users.ts", {
                ...server,
                requiredWrapperCallee: "wrap",
              }),
              lintCase(localAlias, "backend/users.ts", server),
            ],
            invalid: [
              // 6 diagnostics.
              lintCase(
                fixture("server-require-nullable-fetch-wrapper/recall.ts"),
                "backend/users.ts",
                server,
              ),
              lintCase(
                fixture("server-require-nullable-fetch-wrapper/forwarded-this.ts"),
                "backend/users.ts",
                server,
              ),
              lintCase(localGetter, "backend/users.ts", {
                getterCalleePatterns: ["^getUser$"],
                requiredWrapperCallee: "nullableEntity",
              }),
              lintCase(localGetter, "backend/users.ts", {
                getterCalleePatterns: ["^lookup$"],
                requiredWrapperCallee: "nullableEntity",
              }),
              lintCase(
                fixture("server-require-nullable-fetch-wrapper/local-getter-alias.ts"),
                "backend/users.ts",
                {
                  getterCalleePatterns: ["^getUser$"],
                  requiredWrapperCallee: "wrap",
                },
              ),
              // 2 diagnostics.
              lintCase(
                fixture("server-require-nullable-fetch-wrapper/named-call.ts"),
                "backend/users.ts",
                {
                  getterCalleePatterns: ["^rpc\\.call$"],
                  requiredWrapperCallee: "nullableEntity",
                },
              ),
            ],
          },
        ),
      }),
    ).toMatchFileSnapshot("snapshots/recall-call-bindings-remaining.snap");
  }, 30_000);
});

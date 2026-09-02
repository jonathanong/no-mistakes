import tsParser from "@typescript-eslint/parser";
import { RuleTester } from "eslint";
import { describe, it } from "vitest";
import { plugin } from "./helpers.mjs";

const rule = plugin.rules["playwright-no-hoisted-unique-token"];

RuleTester.describe = describe;
RuleTester.it = it;

const ruleTester = new RuleTester({
  languageOptions: { ecmaVersion: 2024, sourceType: "module" },
});

const TOKEN_FACTORIES = [{ tokenFactories: ["randomSuffix"] }];

ruleTester.run("playwright-no-hoisted-unique-token", rule, {
  valid: [
    // No options configured: the rule is inert regardless of shape.
    {
      code: `const suffix = randomSuffix();
      test.beforeAll(async () => {
        await createPost({ slug: \`post-\${suffix}\` });
      });`,
      filename: "playwright/tests/tags/tag-limit.spec.mts",
    },
    // Non-Playwright path: the rule never activates outside e2e/playwright specs.
    {
      code: `const suffix = randomSuffix();
      test.beforeAll(async () => {
        await createPost({ slug: \`post-\${suffix}\` });
      });`,
      filename: "src/util.ts",
      options: TOKEN_FACTORIES,
    },
    // Post-fix tag-limit.spec.mts shape: the factory call is the first statement inside the hook.
    {
      code: `let suffix = "";
      test.beforeAll(async () => {
        suffix = randomSuffix();
        await createPost({ slug: \`post-\${suffix}\` });
      });`,
      filename: "playwright/tests/tags/tag-limit.spec.mts",
      options: TOKEN_FACTORIES,
    },
    // Post-fix content-language.spec.mts shape: describe-scope `let`, assigned inside the hook.
    {
      code: `test.describe("Content Language Rendering", () => {
        let suffix = "";
        test.beforeAll(async () => {
          suffix = randomSuffix();
          await createPost({ slug: \`post-\${suffix}\` });
        });
      });`,
      filename: "playwright/tests/posts/content-language.spec.mts",
      options: TOKEN_FACTORIES,
    },
    // community-features.spec.mts / lists.spec.mts shape: never hoisted — every `suffix` is
    // already the first statement of its own independent beforeAll. A naive earlier-in-file
    // text match would wrongly correlate the second block's `suffix` with the first block's
    // declaration; scope resolution does not.
    {
      code: `test.describe("Block one", () => {
        test.beforeAll(async () => {
          const suffix = randomSuffix();
          await createCommunity({ slug: \`community-\${suffix}\` });
        });
      });
      test.describe("Block two", () => {
        test.beforeAll(async () => {
          const suffix = randomSuffix();
          await createCommunity({ slug: \`community-\${suffix}\` });
        });
      });`,
      filename: "playwright/tests/communities/community-features.spec.mts",
      options: TOKEN_FACTORIES,
    },
    // appeals.spec.mts / modmail.spec.mts / direct-messages.spec.mts shape: already declared
    // directly inside the hook, never hoisted.
    {
      code: `test.beforeAll(async () => {
        const suffix = randomSuffix();
        await createReport({ reason: \`reason-\${suffix}\` });
      });`,
      filename: "playwright/tests/admin/appeals.spec.mts",
      options: TOKEN_FACTORIES,
    },
    // beforeEach is deliberately out of scope: it re-runs by design and collides loudly and
    // deterministically on the very next test, so it isn't the silent hazard this rule targets.
    {
      code: `const suffix = randomSuffix();
      test.beforeEach(async () => {
        await createPost({ slug: \`post-\${suffix}\` });
      });`,
      filename: "playwright/tests/posts/post-lock.spec.mts",
      options: TOKEN_FACTORIES,
    },
    // A factory call hoisted but never read inside any beforeAll at all.
    {
      code: `const suffix = randomSuffix();
      test("uses suffix directly", async () => {
        await createPost({ slug: \`post-\${suffix}\` });
      });`,
      filename: "playwright/tests/posts/post-lock.spec.mts",
      options: TOKEN_FACTORIES,
    },
    // An unconfigured factory name is never tracked.
    {
      code: `const suffix = otherFactory();
      test.beforeAll(async () => {
        await createPost({ slug: \`post-\${suffix}\` });
      });`,
      filename: "playwright/tests/posts/post-lock.spec.mts",
      options: TOKEN_FACTORIES,
    },
    // Destructuring and non-factory initializers are never candidates.
    {
      code: `const { suffix } = randomSuffix();
      const other = 1;
      test.beforeAll(async () => {
        await createPost({ slug: \`post-\${suffix}\`, other });
      });`,
      filename: "playwright/tests/posts/post-lock.spec.mts",
      options: TOKEN_FACTORIES,
    },
    // The hook reassigns the hoisted variable before reading it — the declarator is a candidate
    // (its own initializer is a factory call), but every read inside the hook happens after the
    // in-hook reassignment refreshes it, so nothing is stale.
    {
      code: `let suffix = randomSuffix();
      test.beforeAll(async () => {
        suffix = await computeSuffix();
        await createPost({ slug: \`post-\${suffix}\` });
      });`,
      filename: "playwright/tests/posts/post-lock.spec.mts",
      options: TOKEN_FACTORIES,
    },
    // A declaration nested inside a local helper function that is itself defined and invoked
    // inside beforeAll is minted fresh on every call, however many function boundaries separate
    // it from the hook.
    {
      code: `test.beforeAll(async () => {
        async function setup() {
          const suffix = randomSuffix();
          await createPost({ slug: \`post-\${suffix}\` });
        }
        await setup();
      });`,
      filename: "playwright/tests/posts/post-lock.spec.mts",
      options: TOKEN_FACTORIES,
    },
    // Non-Playwright path activated only via the @playwright/test import, with no hoisted read.
    {
      code: `import { test } from "@playwright/test";
      const suffix = randomSuffix();
      test("uses suffix directly", async () => {
        await createPost({ slug: \`post-\${suffix}\` });
      });`,
      filename: "src/util.ts",
      options: TOKEN_FACTORIES,
    },
    // An unrelated import does not activate a non-Playwright path.
    {
      code: `import { randomSuffix } from "./factories";
      const suffix = randomSuffix();
      test.beforeAll(async () => {
        await createPost({ slug: \`post-\${suffix}\` });
      });`,
      filename: "src/util.ts",
      options: TOKEN_FACTORIES,
    },
    // A `typeof` type query referencing a hoisted candidate is a type-only position, not a
    // runtime read — it never observes the stale hoisted value and must not be flagged.
    {
      code: `const suffix = randomSuffix();
      test.beforeAll(async () => {
        type Suffix = typeof suffix;
        const value: Suffix = "x";
        await createPost({ slug: \`post-\${value}\` });
      });`,
      filename: "playwright/tests/posts/post-lock.spec.mts",
      options: TOKEN_FACTORIES,
      languageOptions: { parser: tsParser },
    },
  ],
  invalid: [
    // Exact pre-fix tag-limit.spec.mts / post-lock.spec.mts shape: module-scope hoist.
    {
      code: `const suffix = randomSuffix();
      test.beforeAll(async () => {
        await createPost({ slug: \`post-\${suffix}\` });
      });`,
      filename: "playwright/tests/tags/tag-limit.spec.mts",
      options: TOKEN_FACTORIES,
      errors: [{ messageId: "hoisted", data: { name: "suffix", factory: "randomSuffix" } }],
    },
    // Exact pre-fix content-language.spec.mts shape: describe-scope hoist.
    {
      code: `test.describe("Content Language Rendering", () => {
        const suffix = randomSuffix();
        test.beforeAll(async () => {
          await createPost({ slug: \`post-\${suffix}\` });
        });
      });`,
      filename: "playwright/tests/posts/content-language.spec.mts",
      options: TOKEN_FACTORIES,
      errors: [{ messageId: "hoisted", data: { name: "suffix", factory: "randomSuffix" } }],
    },
    // Bare (unqualified) beforeAll is also recognized.
    {
      code: `const suffix = randomSuffix();
      beforeAll(async () => {
        await createPost({ slug: \`post-\${suffix}\` });
      });`,
      filename: "playwright/tests/tags/tag-limit.spec.mts",
      options: TOKEN_FACTORIES,
      errors: [{ messageId: "hoisted", data: { name: "suffix", factory: "randomSuffix" } }],
    },
    // Two independent beforeAll hooks each reading the same hoisted token: reported once per read.
    {
      code: `const suffix = randomSuffix();
      test.beforeAll(async () => {
        await createPost({ slug: \`post-\${suffix}\` });
      });
      test.beforeAll(async () => {
        await createComment({ body: \`comment-\${suffix}\` });
      });`,
      filename: "playwright/tests/tags/tag-limit.spec.mts",
      options: TOKEN_FACTORIES,
      errors: [
        { messageId: "hoisted", data: { name: "suffix", factory: "randomSuffix" } },
        { messageId: "hoisted", data: { name: "suffix", factory: "randomSuffix" } },
      ],
    },
    // Multiple hoisted tokens read by the same hook are each reported.
    {
      code: `const suffix = randomSuffix();
      const otherSuffix = randomSuffix();
      test.beforeAll(async () => {
        await createPost({ slug: \`post-\${suffix}\`, tag: otherSuffix });
      });`,
      filename: "playwright/tests/tags/tag-limit.spec.mts",
      options: [{ tokenFactories: ["randomSuffix"] }],
      errors: [
        { messageId: "hoisted", data: { name: "suffix", factory: "randomSuffix" } },
        { messageId: "hoisted", data: { name: "otherSuffix", factory: "randomSuffix" } },
      ],
    },
    // A non-computed member-expression property (`config.suffix`) sharing the hoisted token's
    // name is a binding position, not a reference — it must not be double-reported alongside
    // the real read of the hoisted `suffix`.
    {
      code: `const suffix = randomSuffix();
      test.beforeAll(async () => {
        const config = {};
        config.suffix = "unrelated";
        await createPost({ slug: \`post-\${suffix}\` });
      });`,
      filename: "playwright/tests/tags/tag-limit.spec.mts",
      options: TOKEN_FACTORIES,
      errors: [{ messageId: "hoisted", data: { name: "suffix", factory: "randomSuffix" } }],
    },
    // describe.each(...)('...', fn) shape: fn's enclosing call has a CallExpression callee
    // (`describe.each([...])`), neither a bare Identifier nor a non-computed MemberExpression.
    // That callee shape is not a recognized shield, so the declaration stays describe-scoped
    // and a beforeAll reading it must still be flagged.
    {
      code: `describe.each(["a", "b"])("%s", () => {
        const suffix = randomSuffix();
        test.beforeAll(async () => {
          await createPost({ slug: \`post-\${suffix}\` });
        });
      });`,
      filename: "playwright/tests/tags/tag-limit.spec.mts",
      options: TOKEN_FACTORIES,
      errors: [{ messageId: "hoisted", data: { name: "suffix", factory: "randomSuffix" } }],
    },
    // test.describe.only(...) / .skip(...) still has "describe" in its callee chain — it
    // registers once, exactly like plain describe, and must not be misread as a safe test-body
    // shield just because "only"/"skip" is the last property.
    {
      code: `test.describe.only("Content Language Rendering", () => {
        const suffix = randomSuffix();
        test.beforeAll(async () => {
          await createPost({ slug: \`post-\${suffix}\` });
        });
      });`,
      filename: "playwright/tests/posts/content-language.spec.mts",
      options: TOKEN_FACTORIES,
      errors: [{ messageId: "hoisted", data: { name: "suffix", factory: "randomSuffix" } }],
    },
    // A file path that doesn't match the naming convention is still recognized once it imports
    // @playwright/test.
    {
      code: `import { test } from "@playwright/test";
      const suffix = randomSuffix();
      test.beforeAll(async () => {
        await createPost({ slug: \`post-\${suffix}\` });
      });`,
      filename: "src/util.ts",
      options: TOKEN_FACTORIES,
      errors: [{ messageId: "hoisted", data: { name: "suffix", factory: "randomSuffix" } }],
    },
    // A `suffix as string` cast is a runtime value position (the value-wrapper recurses into its
    // own `.expression`), so the read underneath it is still a real, stale read and must still
    // be flagged.
    {
      code: `const suffix = randomSuffix();
      test.beforeAll(async () => {
        await createPost({ slug: \`post-\${suffix as string}\` });
      });`,
      filename: "playwright/tests/posts/post-lock.spec.mts",
      options: TOKEN_FACTORIES,
      languageOptions: { parser: tsParser },
      errors: [{ messageId: "hoisted", data: { name: "suffix", factory: "randomSuffix" } }],
    },
  ],
});

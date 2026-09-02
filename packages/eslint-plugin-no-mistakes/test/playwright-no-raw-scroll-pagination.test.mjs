import { RuleTester } from "eslint";
import { describe, it } from "vitest";
import { plugin } from "./helpers.mjs";

const rule = plugin.rules["playwright-no-raw-scroll-pagination"];

RuleTester.describe = describe;
RuleTester.it = it;

const ruleTester = new RuleTester({
  languageOptions: { ecmaVersion: 2024, sourceType: "module" },
});

const PAGINATED_WAIT = `page.waitForRequest(
  (req) => req.url().includes("/api/v1/posts") && req.url().includes("after=")
);`;

ruleTester.run("playwright-no-raw-scroll-pagination", rule, {
  valid: [
    // Non-Playwright path: the rule is inert outside e2e/playwright specs.
    {
      code: `${PAGINATED_WAIT}\nawait page.evaluate(() => window.scrollTo(0, document.body.scrollHeight));`,
      filename: "src/util.ts",
    },
    // Raw scrollTo with no paginated wait anywhere in the file — scroll-to-top style tests.
    {
      code: "await page.evaluate(() => window.scrollTo(0, 0));",
      filename: "playwright/feed/news-discuss-scroll.spec.mts",
    },
    // Cursor wait present but driven with the repeated-scroll helper, not a raw scroll.
    {
      code: `${PAGINATED_WAIT}\nawait scrollToLoadMore(page);`,
      filename: "playwright/reviews.spec.mts",
    },
    // Cursor wait present but the raw scroll param does not match any configured cursor param.
    {
      code: `page.waitForRequest((req) => req.url().includes("/api/v1/posts?page=2"));\nawait page.evaluate(() => window.scrollTo(0, 0));`,
      filename: "playwright/reviews.spec.mts",
    },
    // waitForRequest with no first argument at all — nothing to inspect for a cursor param.
    {
      code: "page.waitForRequest();\nawait page.evaluate(() => window.scrollTo(0, 0));",
      filename: "playwright/reviews.spec.mts",
    },
    // scrollIntoView is element-relative, not a pagination driver, and is never matched.
    {
      code: `${PAGINATED_WAIT}\nawait locator.scrollIntoView();`,
      filename: "playwright/reviews.spec.mts",
    },
  ],
  invalid: [
    // The exact pre-fix reviews.spec.mts shape: waitForRequest predicate mentions `after=`,
    // scrolling is a raw window.scrollTo inside page.evaluate.
    {
      code: `${PAGINATED_WAIT}\nawait page.evaluate(() => window.scrollTo(0, document.body.scrollHeight));`,
      filename: "playwright/reviews.spec.mts",
      errors: [{ messageId: "rawScroll" }],
    },
    // waitForResponse variant, matching on the response URL's search params.
    {
      code: `page.waitForResponse(
        (response) => response.url().includes("/api/v1/posts") && response.status() === 200 && new URL(response.url()).searchParams.has("after")
      );
      await page.evaluate(() => window.scrollTo(0, document.body.scrollHeight));`,
      filename: "playwright/reviews.spec.mts",
      errors: [{ messageId: "rawScroll" }],
    },
    // Bare scrollBy, and the cursor param mentioned via a regex literal.
    {
      code: `page.waitForRequest((req) => /after=/.test(req.url()));\nawait page.evaluate(() => scrollBy(0, 200));`,
      filename: "e2e/reviews.spec.ts",
      errors: [{ messageId: "rawScroll" }],
    },
    // Cursor param mentioned via a template literal quasi.
    {
      code: "page.waitForRequest((req) => req.url().includes(`after=${cursor}`));\nawait page.evaluate(() => window.scrollTo(0, 0));",
      filename: "playwright/reviews.spec.mts",
      errors: [{ messageId: "rawScroll" }],
    },
    // Cursor param mentioned via a plain string literal passed directly (no predicate function).
    {
      code: 'page.waitForRequest("**/api/v1/posts?after=abc");\nawait page.evaluate(() => window.scrollTo(0, 0));',
      filename: "playwright/reviews.spec.mts",
      errors: [{ messageId: "rawScroll" }],
    },
    // Two raw scrolls in one file with a single qualifying wait — both are reported.
    {
      code: `${PAGINATED_WAIT}\nawait page.evaluate(() => window.scrollTo(0, 0));\nawait page.evaluate(() => window.scrollBy(0, 100));`,
      filename: "playwright/reviews.spec.mts",
      errors: [{ messageId: "rawScroll" }, { messageId: "rawScroll" }],
    },
    // Custom cursorParams option: "page" is not in the default set.
    {
      code: 'page.waitForRequest((req) => req.url().includes("/api/v1/posts?page=2"));\nawait page.evaluate(() => window.scrollTo(0, 0));',
      filename: "playwright/reviews.spec.mts",
      options: [{ cursorParams: ["page"] }],
      errors: [{ messageId: "rawScroll" }],
    },
    // scrollHelper option is interpolated into the message.
    {
      code: `${PAGINATED_WAIT}\nawait page.evaluate(() => window.scrollTo(0, document.body.scrollHeight));`,
      filename: "playwright/reviews.spec.mts",
      options: [{ scrollHelper: "scrollToLoadMore" }],
      errors: [
        {
          messageId: "rawScroll",
          data: {
            helperHint:
              " Use scrollToLoadMore, which scrolls repeatedly until the request fires, instead.",
          },
        },
      ],
    },
    // .pw.spec path naming convention is also recognized as a Playwright file.
    {
      code: `${PAGINATED_WAIT}\nawait page.evaluate(() => window.scrollTo(0, document.body.scrollHeight));`,
      filename: "reviews.pw.spec.ts",
      errors: [{ messageId: "rawScroll" }],
    },
  ],
});

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
    // A cursor param name that appears only as a substring of an unrelated query value
    // (`after-hours`, not `after=`) must not satisfy the boundary match.
    {
      code: 'page.waitForRequest((req) => req.url().includes("category=after-hours"));\nawait page.evaluate(() => window.scrollTo(0, 0));',
      filename: "playwright/reviews.spec.mts",
      options: [{ cursorParams: ["after"] }],
    },
    // A same-named method on a non-window receiver (a map widget, an editor) is not a browser
    // scroll and is never matched, regardless of a qualifying cursor wait elsewhere in the file.
    {
      code: `${PAGINATED_WAIT}\nmap.scrollTo({ lat: 0, lng: 0 });\nawait editor.scrollTo(0, 0);`,
      filename: "playwright/reviews.spec.mts",
    },
    // An unrelated import does not activate a non-Playwright path.
    {
      code: `import { scrollToLoadMore } from "./helpers";\n${PAGINATED_WAIT}\nawait page.evaluate(() => window.scrollTo(0, document.body.scrollHeight));`,
      filename: "src/util.ts",
    },
    // A non-literal .searchParams.has(...) argument is never treated as a cursor-param match.
    {
      code: `page.waitForRequest((req) => new URL(req.url()).searchParams.has(cursorParamName));\nawait page.evaluate(() => window.scrollTo(0, 0));`,
      filename: "playwright/reviews.spec.mts",
    },
    // A locally-declared `scrollTo` helper resolves to its own declaration, not the global —
    // calling it bare must not be misflagged as the raw browser API.
    {
      code: `function scrollTo() {}\n${PAGINATED_WAIT}\nawait scrollTo();`,
      filename: "playwright/reviews.spec.mts",
    },
    // A locally-imported `scroll` helper is likewise a different function from the global.
    {
      code: `import { scroll } from "./helpers";\n${PAGINATED_WAIT}\nawait scroll();`,
      filename: "playwright/reviews.spec.mts",
    },
    // A regex whose alternation happens to close right before an unrelated compound param name
    // (`(?:next|prev)cursor=` matches `nextcursor=`/`prevcursor=`, not a `cursor` query key) must
    // not be treated as mentioning the configured `cursor` param just because a `)` precedes it —
    // that `)` closes an alternation with nothing to do with a query-string separator.
    {
      code: "page.waitForRequest((req) => /(?:next|prev)cursor=/.test(req.url()));\nawait page.evaluate(() => window.scrollTo(0, 0));",
      filename: "playwright/reviews.spec.mts",
      options: [{ cursorParams: ["cursor"] }],
    },
    // A plain (non-regex) literal where the param name appears mid-string with no `?`/`&`
    // immediately before it (`pageafter=2`, not `?after=2`) must not satisfy the boundary check —
    // the same non-boundary rule that applies to regex literals also applies to plain strings.
    {
      code: 'page.waitForRequest((req) => req.url().includes("pageafter=2"));\nawait page.evaluate(() => window.scrollTo(0, 0));',
      filename: "playwright/reviews.spec.mts",
      options: [{ cursorParams: ["after"] }],
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
    // window.scroll(...) is the same imperative position-based API as scrollTo, just under its
    // other name, and must be matched too.
    {
      code: `${PAGINATED_WAIT}\nawait page.evaluate(() => window.scroll(0, document.body.scrollHeight));`,
      filename: "playwright/reviews.spec.mts",
      errors: [{ messageId: "rawScroll" }],
    },
    // A bare, unshadowed `scroll()` call resolves to the global and is matched the same as
    // scrollTo/scrollBy.
    {
      code: `${PAGINATED_WAIT}\nawait page.evaluate(() => scroll(0, 0));`,
      filename: "playwright/reviews.spec.mts",
      errors: [{ messageId: "rawScroll" }],
    },
    // The cursor param is mentioned via a regex literal whose own boundary syntax
    // (`[?&]after=`) leaves a `]` immediately before the param name in the pattern *source* —
    // matched as text, not executed, so the boundary check must recognize `]` (and `)`, for
    // constructs like `(?:^|[?&])after=`) as a boundary specifically for regex-derived literals.
    {
      code: `page.waitForRequest((req) => /[?&]after=/.test(req.url()));\nawait page.evaluate(() => window.scrollTo(0, 0));`,
      filename: "playwright/reviews.spec.mts",
      errors: [{ messageId: "rawScroll" }],
    },
    // A file path that doesn't match the naming convention is still recognized once it imports
    // @playwright/test.
    {
      code: `import { test } from "@playwright/test";\n${PAGINATED_WAIT}\nawait page.evaluate(() => window.scrollTo(0, document.body.scrollHeight));`,
      filename: "src/util.ts",
      errors: [{ messageId: "rawScroll" }],
    },
  ],
});

import type { test as TypeOnlyTest } from "@playwright/test";
import { expect, test, type TestType } from "@playwright/test";

const contextMethod = "newContext";

test("uses shared page fixture", async ({ page }) => {
  await page.goto("/orders");
  await expect(page.getByRole("heading")).toBeVisible();
});

test("uses isolated browser context", async ({ browser }) => {
  const context = await browser.newContext();
  const page = await context.newPage();
  await page.goto("/public");
  await context.close();
  await browser[contextMethod]();
  const { newContext } = browser;
  await newContext();
  if (browser && browser.isConnected()) {
    expect(browser.version()).toBeTruthy();
  }
  const ok = !browser || typeof browser === "object" || browser !== null ? true : false;
  void ok;
  browser && browser.version();
  browser || browser.version();
  await browser.newContext.call(browser);
});

test("fixtures object reads non-browser keys and safe checks", async (fixtures) => {
  if (fixtures) {
    await fixtures.page.goto("/orders");
  }
  const { page } = fixtures;
  await page.reload();
});

test("extracts browser before rest fixture destructuring", async ({ browser, ...rest }) => {
  await browser.newContext();
  void rest;
});

test("no parameters", async () => {
  expect(1).toBe(1);
});

test("ts this parameter only", async function (this: unknown) {
  expect(this).toBeUndefined();
});

test.describe("suite callback does not receive fixtures", () => {
  test.describe.only("nested suite", () => {
    const browser = { newPage: () => ({}) };
    browser.newPage();
  });
});

test.use({ viewport: { width: 1280, height: 720 } });
test.only.skip(true, "chained modifier annotation");
test.skip(true, "conditional skip annotation");
test.skip(true, `conditional skip ${contextMethod}`);
test.skip(true);
test.skip();
test.beforeEach();
test("single arg call");
const [localRunner] = [() => {}];
localRunner();
const helpers = { a: { run: () => {} }, go: () => {} };
const {
  a: { run },
} = helpers;
run();
const { a: _nested, ...restHelpers } = helpers;
restHelpers.go();
let localCallback = () => {};
localCallback = () => {};
localCallback();

const customTest = test.extend({
  appPage: async ({ browser }, use) => {
    const page = await browser.newPage();
    await use(page);
  },
});
void customTest;

function unrelatedHelper({ browser }: { browser: { newPage(): Promise<void> } }) {
  return browser.newPage();
}
void unrelatedHelper;
export type { TypeOnlyTest, TestType };

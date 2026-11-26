import pwDefault, { mergeTests, test, "test" as quotedTest } from "@playwright/test";
import * as pw from "@playwright/test";

const methodKey = "newPage";

test("direct call", async ({ browser }) => {
  const page = await browser.newPage();
  await page.goto("/orders");
});

test("optional chain, bracket, cast, and const key", async ({ browser }) => {
  await browser?.newPage?.();
  await browser["newPage"]();
  await browser[`newPage`]();
  await (browser as { newPage(): Promise<void> }).newPage();
  await browser[methodKey]();
});

test("renamed fixture and local alias", async ({ "browser": localBrowser = fallbackBrowser }) => {
  const alias = localBrowser;
  await alias.newPage();
});

test("destructured newPage and aliases", async ({ browser }) => {
  const { newPage } = browser;
  await newPage();
  const { newPage: openPage = fallbackOpen } = browser;
  await openPage();
  const detached = browser.newPage;
  await detached();
  const bound = browser.newPage.bind(browser);
  await bound();
});

test("call, apply, bind, and escaping references", async ({ browser }) => {
  await browser.newPage.call(browser);
  await browser.newPage.apply(browser, []);
  await browser.newPage.bind(browser)();
  const unboundCall = browser.newPage.call;
  const methodName = browser.newPage.name;
  runHelper(browser.newPage);
  const { newPage: unusedNewPage } = browser;
  let reassigned = browser.newPage;
  reassigned = fallbackOpen;
  const { newPage: { length: nested } } = browser;
  void [unboundCall, methodName, reassigned, nested];
});

test("raw fixtures parameter and nested destructuring", async (fixtures) => {
  const bag = fixtures;
  await bag.browser.newPage();
  const { browser: fromBag } = bag;
  await fromBag.newPage();
});

test("nested browser destructuring in parameter", async ({ browser: { newPage } }) => {
  await newPage();
});

test.beforeEach(async ({ browser }) => {
  await browser.newPage();
});

test.only("focused test", async ({ browser }) => {
  const open = () => browser.newPage();
  await open();
});

test.skip("skipped test", async ({ browser }) => {
  await browser.newPage();
});

test.fixme(async ({ browser }) => {
  await browser.newPage();
});

const extended = test.extend({});
extended("extended test", async ({ browser }) => {
  await browser.newPage();
});

const merged = mergeTests(test, extended);
merged("merged test", async ({ browser }) => {
  await browser.newPage();
});

pw.test("namespace test", async ({ browser }) => {
  await browser.newPage();
});

pwDefault("default import test", async ({ browser }) => {
  await browser.newPage();
});

async function namedCallback({ browser }: { browser: { newPage(): Promise<void> } }) {
  const { newPage } = browser;
  await newPage();
}
const aliasedCallback = namedCallback;
test("named callback", aliasedCallback);
quotedTest("receiver-bound callback", namedCallback.bind(null));

const { only: runOnly } = test;
runOnly("destructured modifier", async ({ browser }) => {
  await browser.newPage();
});

test("ts this parameter", async function (this: unknown, { browser }) {
  await browser.newPage();
});

test("repeated var newPage self-alias does not recurse", async ({ browser }) => {
  var create = browser.newPage;
  var create = create;
  void create;
});

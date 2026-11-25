import { test } from "@playwright/test";

declare const dynamicKey: string;
declare const dynamicMethod: string;
declare const externalCallback: () => Promise<void>;
declare const spreadArgs: [string, () => Promise<void>];
declare const openViaHelper: (arg: unknown) => Promise<void>;
declare const foreignHelper: { run(this: unknown): Promise<void> };

test("dynamic member on browser", async ({ browser }) => {
  await browser[dynamicKey]();
});

test("dynamic destructure and rest on browser", async ({ browser }) => {
  const { [dynamicKey]: open } = browser;
  const { ...rest } = browser;
  const [tupleItem] = browser as unknown as [unknown];
  void [open, rest, tupleItem];
});

test("browser escapes and reassigned alias", async ({ browser }) => {
  await openViaHelper(browser);
  await foreignHelper.run.call(browser);
  let alias = browser;
  alias = {} as typeof browser;
  void alias;
});

test("rest and dynamic fixture destructuring", async ({ ...rest }) => {
  void rest;
});

test("dynamic fixture key", async ({ [dynamicKey]: item }) => {
  void item;
});

test("unsupported browser pattern in fixtures", async ({ browser: [first] }: { browser: unknown[] }) => {
  void first;
});

test("rest parameter captures fixtures", async (...args) => {
  void args;
});

test("array pattern parameter", async ([first] = []) => {
  void first;
});

test("raw fixtures escapes, dynamic access, and reassignment", async (fixtures) => {
  await openViaHelper(fixtures);
  await fixtures[dynamicKey]();
  const [item] = fixtures as unknown as [unknown];
  void item;
});

test("reassigned fixtures binding", async (fixtures) => {
  fixtures = {} as typeof fixtures;
  void fixtures;
});

test("unresolved external callback", externalCallback);
function reassignedFn() {}
reassignedFn = externalCallback;
test("reassigned function callback", reassignedFn);
test("partially bound callback", ((a: unknown, b: unknown) => [a, b]).bind(null, 1));
test("spread bound callback", ((a: unknown) => a).bind(...spreadArgs));
test("spread test arguments", ...spreadArgs);
test.beforeEach(...spreadArgs);
test.beforeEach(externalCallback);
test.skip("skipped with unknown callback", externalCallback);
test.skip(...spreadArgs);
test.skip("title", ...spreadArgs);
test[dynamicMethod]("dynamic registrar", async ({ browser }) => {
  await browser.newContext();
});

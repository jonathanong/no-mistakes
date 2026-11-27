import { expect, test } from "@example/e2e-fixtures";
import { test as unconfiguredTest } from "./unconfigured-fixtures";
import fs = require("node:fs");
import testEquals = localBinding.playwright;

declare const externalCallback: () => Promise<void>;
declare const localBinding: {
  playwright: (name: string, fn: () => Promise<void>) => void;
};

test("loads orders", async ({ browser }) => {
  await browser.newPage();
});

unconfiguredTest("alias from another module", async ({ browser }) => {
  await browser.newPage();
});

test("shared page stays quiet", async ({ page }) => {
  await page.goto("/orders");
});

test("external callback is untraceable", externalCallback);

test("rest fixtures are untraceable", async ({ ...rest }) => {
  await rest.page.goto("/orders");
});

testEquals("qualified import equals stays quiet", async ({ browser }) => {
  await browser.newPage();
});

expect(1);

fs("loads orders", async ({ browser }: { browser: { newPage(): Promise<void> } }) => {
  await browser.newPage();
});

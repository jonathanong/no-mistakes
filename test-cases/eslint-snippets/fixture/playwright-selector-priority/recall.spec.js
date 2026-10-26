import { test } from "@playwright/test";

page["locator"]("#save");
page[`locator`](".save");
(page?.locator)("h1");
const { locator } = page;
locator("text=Save");
page.locator.call(page, "#save");
(0, page.locator)(".save");
page[name]("#save");
void test;

// A same-named parameter and its alias are not the page locator method.
function unrelated(locator) {
  locator(".save");
  const alias = locator;
  alias(".save");
}
function renamedParameter(callback) {
  const locator = callback;
  locator(".save");
}

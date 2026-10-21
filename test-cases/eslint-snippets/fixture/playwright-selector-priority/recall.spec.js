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

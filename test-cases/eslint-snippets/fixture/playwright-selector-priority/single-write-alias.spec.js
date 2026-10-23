import { test } from "@playwright/test";

// Never-reassigned let and var bindings are the member they alias.
let locator = page.locator;
locator(".a");
var find = page.locator;
let inner = find;
inner(".b");
find("#save");
let { locator: named } = page;
named("text=Save");
var { locator: tagged } = page;
tagged("h2");

// A write anywhere in the binding's scope hides every call through it.
let rebound = page.locator;
rebound(".a");
rebound = page.click;
var later = page.locator;
later = page.getByRole;
later("#save");

// A nested pattern is not a method name.
const {
  locator: [nested],
} = page;
nested(".a");

void test;

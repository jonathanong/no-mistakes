import { locator } from "./unrelated.js";
locator(".not-playwright");
function run() {
  function locator() {}
  locator(".not-playwright");
}
function withClass() {
  class locator {}
  locator(".not-playwright");
}
function mutable() {
  // A later write fails closed: the binding is not a static locator.
  let locator = page.locator;
  locator = page.click;
  locator(".not-static");
}
function cyclic() {
  const locator = locator;
  locator(".unknown");
}
function ambiguous() {
  var locator;
  var locator;
  locator(".unknown");
}
// Missing arguments are not selectors.
page.locator();

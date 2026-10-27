import { defineConfig, test } from "@playwright/test";

// Loop bindings have a null initializer. Ordinary computed calls must not throw
// or be reported as Playwright timeouts. A const alias of setTimeout still is.
const plugin = {};
for (const key of ["run"]) {
  plugin[key]();
}
const bag = { run: 1 };
for (const key in bag) {
  plugin[key]();
}
for (const [key] of [["go"]]) {
  plugin[key]();
}
for (const { key } of [{ key: "go" }]) {
  plugin[key]();
}
for (const key of ["run"]) {
  plugin[key] = true;
}
plugin["run"]();

const method = "setTimeout";
test[method](30001);
test.setTimeout(30001);
export default defineConfig({ timeout: 30001 });

for (const key of ["setTimeout"]) {
  test[key](30001);
}

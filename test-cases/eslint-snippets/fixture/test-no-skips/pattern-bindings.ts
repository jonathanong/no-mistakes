const fallback = {};
const dynamic = "test";

const { test: renamed = fallback } = require("vitest");
// no-mistakes-disable-next-line test-no-skips -- defaulted rename is still the test binding
renamed.skip("suppressed default", () => {});
renamed.only("defaulted rename", () => {});

const { it = fallback } = require("vitest");
it.todo("defaulted shorthand", () => {});

const { ["describe"]: quoted = fallback } = require("vitest");
quoted.fixme("defaulted literal key", () => {});

const { test: awaited = fallback } = await import("vitest");
awaited.skip("awaited default", () => {});

const { runner: { test: nested = fallback } = fallback } = require("@playwright/test");
nested.fixme("nested default", () => {});

const { group: { describe: nestedDescribe } } = require("vitest");
nestedDescribe.skip("nested pattern", () => {});

const { test: { skip: skipped = fallback }, ...rest } = require("vitest");
skipped.only("destructured modifier is not a test binding", () => {});
rest.skip("rest is not a test binding", () => {});

const { [dynamic]: computed = fallback } = require("vitest");
computed.skip("dynamic key", () => {});

const { test: ordinary = fallback } = require("ordinary");
ordinary.skip("unrelated module", () => {});

let { test: rebound = fallback } = require("vitest");
rebound.skip("let binding", () => {});

const { other: { name: label }, test: sibling = fallback } = require("vitest");
label.skip("nested non-test", () => {});
sibling.runIf(true)("sibling after a nested miss", () => {});

const { rows: [row] = [] } = require("vitest");
row.skip("array element", () => {});

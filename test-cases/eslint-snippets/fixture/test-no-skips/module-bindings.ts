const { test: requiredTest, describe: requiredDescribe } = require("vitest");
requiredTest.skip("CommonJS named alias", () => {});
requiredDescribe.skip("CommonJS describe alias", () => {});
const { ["test"]: literalRequiredTest } = require("vitest");
literalRequiredTest.only("CommonJS literal computed name", () => {});

const playwright = require("@playwright/test");
playwright.test.fixme("CommonJS namespace", () => {});

const { it: dynamicIt } = await import("vitest");
dynamicIt.todo("dynamic import named alias", () => {});

const dynamicPlaywright = await import("@playwright/test");
dynamicPlaywright.test.fixme("dynamic import namespace", () => {});

const { test: ordinaryTest } = require("ordinary");
ordinaryTest.skip("unrelated CommonJS module", () => {});

const ordinary = await import("ordinary");
ordinary.test.skip("unrelated dynamic module", () => {});
const unresolvedDynamicImport = import("vitest");
unresolvedDynamicImport.test.skip("unawaited dynamic import", () => {});

const dynamicName = "test";
const { [dynamicName]: computedRequiredTest } = require("vitest");
computedRequiredTest.skip("computed nonliteral binding", () => {});

function localRequire(require) {
  const { test: shadowedTest } = require("vitest");
  shadowedTest.skip("shadowed require", () => {});
}

const local = { test: { skip() {} } };
local.test.skip("local lookalike", () => {});

let reboundRequire = require("vitest");
reboundRequire = { test: { skip() {} } };
reboundRequire.test.skip("reassigned lookalike", () => {});

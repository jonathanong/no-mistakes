import { test as base } from "@playwright/test";

class DeclaredSuite {
  runner = base.extend({});
  other = { describe: { serial: (_name: string, _fn: () => void) => {} } };
}

let declared = new DeclaredSuite();
declared.runner.describe.serial("declared class instance", () => {});

const ExprSuite = class {
  runner = base.extend({});
  other = { describe: { serial: (_name: string, _fn: () => void) => {} } };
};

const exprInstance = new ExprSuite();
exprInstance.other.describe.serial("unrelated expr field", () => {});

new (class {
  runner = base;
})().runner.describe.serial("inline class expr", () => {});

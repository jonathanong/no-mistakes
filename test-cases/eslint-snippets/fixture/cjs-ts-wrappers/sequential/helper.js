const vt = require("vitest");
vt.test.sequential("creates", () => {});
const { test: t } = require("vitest");
t.sequential("updates", () => {});
const { expect: e } = require("vitest");
e.sequential("not a runner", () => {});
const { describe: cjsDescribe } = require("vitest");
cjsDescribe.sequential("suite", () => {});

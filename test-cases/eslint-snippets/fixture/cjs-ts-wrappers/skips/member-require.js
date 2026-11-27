const check = require("vitest").test;
check.skip("bound", () => {});
require("vitest").test.skip("inline", () => {});
require("@playwright/test").it.skip("playwright", () => {});
require("vitest")["test"].skip("string key", () => {});

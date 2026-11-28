const vt = require("vitest");
vt.test("slow", () => {}, 999_999);
const { test: t } = require("vitest");
t("slower", () => {}, 999_999);

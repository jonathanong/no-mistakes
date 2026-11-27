const source = "vitest";
const { test: base } = require(source);
const serial = base.sequential;
serial("x", () => {});

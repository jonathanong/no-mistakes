const source = "vitest";
const { test: base } = require(source);
const plain = base;
plain("x", () => {});

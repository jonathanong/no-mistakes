const source = "vitest";
require(source).test.sequential("x", () => {});
require("other").test.sequential("static", () => {});

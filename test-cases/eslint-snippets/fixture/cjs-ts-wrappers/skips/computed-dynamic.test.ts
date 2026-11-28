const specifier = "vitest";
const runner = require(specifier);
runner["test"].skip("case", () => {});

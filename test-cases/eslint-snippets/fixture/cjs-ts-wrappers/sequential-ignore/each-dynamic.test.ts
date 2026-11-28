const source = "vitest";
const { describe: suite } = require(source);
suite.each([[1]])("case %i", { sequential: true }, () => {});

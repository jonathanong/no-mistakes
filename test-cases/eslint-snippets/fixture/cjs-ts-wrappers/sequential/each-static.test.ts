const { describe: suite } = require("vitest");
suite.each([[1]])("case %i", { sequential: true }, () => {});

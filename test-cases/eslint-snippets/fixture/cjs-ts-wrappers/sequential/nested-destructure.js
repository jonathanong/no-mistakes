function register() {
  const { test: suite } = require("vitest");
  suite.sequential("nested alias", () => {});
}
void register;

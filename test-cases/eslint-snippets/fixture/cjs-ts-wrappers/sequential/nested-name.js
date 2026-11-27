function register() {
  const { test: suite } = require("vitest");
}
const suite = { sequential() {} };
suite.sequential();
void register;

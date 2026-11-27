function register() {
  vt.test.sequential("late", () => {});
}
const vt = require("vitest");
register();

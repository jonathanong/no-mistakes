function nested() {
  const vt = require("vitest");
  vt.test.sequential("nested", () => {});
}
const other = {};
const vt = other;
vt.test.sequential("shadow", () => {});
void nested;

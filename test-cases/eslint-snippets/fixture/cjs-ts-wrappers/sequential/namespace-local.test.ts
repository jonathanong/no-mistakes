const vt = require("vitest");
function nested() {
  const local = vt;
  const { test: check } = local;
  check.sequential("nested", () => {});
  local.sequential("nested namespace", () => {});
}

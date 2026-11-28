import vt from "vitest";
const alias = vt;
alias.mock("./dep");
const { mock } = vt;
mock("./dep");
function register() {
  const vt = alias;
  vt.mock("react");
}

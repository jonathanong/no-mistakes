const moduleName = "vitest";
const vt = require(moduleName);
let mock = vt.vi.mock;
mock("./dep");
function nested(mock: (specifier: string) => void) {
  mock("./dep");
}
void nested;

import vt from "vitest";
import { test, vi } from "vitest";
const mockItems: number[] = [];
vt.mock("./dep", () => ({ mockItems }));
const alias = vt;
alias.mock("./other", () => ({ mockItems }));
function register() {
  const vt = alias;
  vt.mock("./shadow", () => ({ mockItems }));
}
void register;
test("mutates", () => {
  mockItems.push(1);
});
const viAlias = vi;
let mockAlias = 0;
viAlias.mock("./kept", () => ({ get: () => mockAlias }));
test("kept", () => {
  mockAlias = 1;
});

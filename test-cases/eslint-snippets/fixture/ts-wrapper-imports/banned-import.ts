import * as evil from "@example/evil";
import eq = require("@example/evil");

const name = "bad";
let g: unknown;

evil["bad" as const]();
const { ["bad" as const]: bad } = evil;
bad();
export const { secret: fromEquals } = eq;
export const { secret: fromRequire } = require("@example/evil");
((() => {
  g = evil.bad;
}) as () => void)();
g();
(require as NodeRequire)("@example/evil").bad();
require("@example/evil" as const).bad();
void name;

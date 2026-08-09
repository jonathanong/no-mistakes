import "../sources/static";

void import("../sources/dynamic");
require("../sources/required");
import "@fixture/runtime";
import "../sources/config.json";

export { named } from "../barrels/named";
export * from "../barrels/star";

import type { OnlyType } from "../sources/type-only";
export type { OnlyType };

import "../sources/static";

void import("../sources/dynamic");
require("../sources/required");

export { named } from "../barrels/named";
export * from "../barrels/star";

import type { OnlyType } from "../sources/type-only";
export type { OnlyType };

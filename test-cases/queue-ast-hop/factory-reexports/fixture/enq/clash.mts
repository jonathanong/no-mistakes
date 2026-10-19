import { createEnqueueFunction } from "../lib.mts";
import { kept as b } from "../origin.mts";

export { exported as b } from "../origin.mts";

export const localUse = createEnqueueFunction(b, "fromImport");

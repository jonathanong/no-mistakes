import { importedTarget } from "./sequence-imported-target.mts";

export function localTarget() {}

(0, localTarget)();
(0, importedTarget)();
(0, dynamicFactory())();

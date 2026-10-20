import { createEnqueueFunction } from "../lib.mts";
import { b } from "./clash.mts";

export const remoteUse = createEnqueueFunction(b, "fromExport");

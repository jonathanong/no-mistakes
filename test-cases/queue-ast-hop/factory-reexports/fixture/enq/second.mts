import { createEnqueueFunction } from "../lib.mts";
import { x } from "../second-a.mts";

export const second = createEnqueueFunction(x, "viaSecondName");

import { createQueue as cq } from "@factory/pkg";

let mutable = "mutable";
const { destructured } = source;
const NON_STRING = 1;
const QUEUE_NAME = "constant";

export const fromConst = cq(QUEUE_NAME);

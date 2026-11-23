import { createQueue } from "./lib/wrappers";

export const objects = createQueue("objects");
export const returned = createQueue("returned");
export const background = createQueue("background");

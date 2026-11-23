import { createQueue } from "./wrappers";

export const partialJobs = createQueue("partial-jobs");
export const partialOptions = createQueue("partial-options");

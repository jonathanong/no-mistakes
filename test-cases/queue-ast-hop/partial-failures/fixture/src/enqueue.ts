import { createEnqueueFunction } from "./factory";
import { queue } from "./queues";

export const enqueue = createEnqueueFunction(queue, "healthy-job");

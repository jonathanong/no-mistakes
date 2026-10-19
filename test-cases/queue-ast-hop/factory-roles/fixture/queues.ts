import { BACKGROUND_QUEUE_NAME } from "./config";
import { createQueue } from "./lib/factory";

export const background = createQueue(BACKGROUND_QUEUE_NAME, {
  priorityKind: "ranked",
});
export const adminImports = background;

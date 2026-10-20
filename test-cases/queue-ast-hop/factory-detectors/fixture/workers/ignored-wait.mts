import { queue } from "../queues.mts";

export async function hidden() {
  await queue.addAndWait("page", {});
}

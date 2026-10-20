import { queue } from "../queues.mts";

export async function route() {
  // no-mistakes-disable-next-line queues-check
  await queue.addAndWait("page", {});
  await queue.addAndWait("other", {});
}

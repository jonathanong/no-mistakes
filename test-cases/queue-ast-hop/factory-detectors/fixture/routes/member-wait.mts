import { queue } from "../queues.mts";

export class Routes {
  queue = queue;

  async handle() {
    await this.queue.addAndWait("page", {});
    await queue.addAndWait("page", {});
    await queue.add("page", {}, { jobId: "static" });
  }
}

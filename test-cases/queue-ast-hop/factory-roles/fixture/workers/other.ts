import { createWorker } from "../lib/factory";
import { background } from "../queues";

export const otherWorker = createWorker("other", async (job) => {
  switch (job.name) {
    case "pong":
      await background.add("page", {});
      return;
  }
});

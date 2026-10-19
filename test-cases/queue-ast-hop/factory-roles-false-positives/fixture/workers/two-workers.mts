import { createWorker } from "../lib.mts";
import { a, b } from "../queues.mts";

export const wa = createWorker("a", async (job) => {
  switch (job.name) {
    case "x":
      await b.add("y", {});
      return;
  }
});

export const wb = createWorker("b", async (job) => {
  switch (job.name) {
    case "y":
      return;
  }
});

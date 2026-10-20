import { createWorker } from "../lib.mts";

export const second = createWorker("visitedTwice", async (job) => {
  switch (job.name) {
    case "viaSecondName":
      return;
  }
});

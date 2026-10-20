import { createWorker } from "../lib.mts";

export const keptWorker = createWorker("keptName", async (job) => {
  switch (job.name) {
    case "fromImport":
      return;
  }
});

export const exportedWorker = createWorker("exportedName", async (job) => {
  switch (job.name) {
    case "fromExport":
      return;
  }
});

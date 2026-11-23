import { createWorker } from "../lib/wrappers";

createWorker("objects", async (job) => {
  switch (job.name) {
    case "objectTask":
    case "directTask":
    case "noDefaultsTask":
      return;
  }
});
createWorker("returned", async (job) => {
  switch (job.name) {
    case "returnedTask":
      return;
  }
});
createWorker("background", async (job) => {
  switch (job.name) {
    case "definitionTask":
    case "bulkTask":
      return;
  }
});

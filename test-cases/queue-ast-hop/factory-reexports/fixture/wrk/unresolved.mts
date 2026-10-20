import { createWorker } from "../lib.mts";

export const nsWorker = createWorker("nsq", async (job) => {
  switch (job.name) {
    case "nsjob":
      return;
  }
});

export const defaultWorker = createWorker("dq", async (job) => {
  switch (job.name) {
    case "dfltjob":
      return;
  }
});

export const externalWorker = createWorker("q6", async (job) => {
  switch (job.name) {
    case "ext":
      return;
  }
});

export const cycleWorker = createWorker("q3", async (job) => {
  switch (job.name) {
    case "cyc":
      return;
  }
});

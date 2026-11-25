import { Queue } from "bullmq";

export function makeEnqueue(job: { name: string }) {
  return (data: object, options?: { removeOnFail: boolean }) =>
    getQueue().add(job.name, data, options);
}

function getQueue() {
  return new Queue("opaque", { defaultJobOptions: { removeOnFail: true } });
}

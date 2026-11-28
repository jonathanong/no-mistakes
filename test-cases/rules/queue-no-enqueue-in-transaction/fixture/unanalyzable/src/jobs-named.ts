import { createQueue as importedCreateQueue } from "@sample/jobs";
import * as importedJobsNs from "@sample/jobs";
export { enqueue, enqueue as send, createQueue } from "@sample/jobs";
export * as jobsNs from "@sample/jobs";
export { importedJobsNs as localJobsNs };
export { enqueue as sendViaStar } from "./jobs-star";
export { enqueue as missingSend } from "./jobs-missing";
export type { Queue, QueueAdder } from "@sample/jobs";
declare function getSender(): (row: { id: string }) => void;
export const opaqueSend = getSender();
export const opaqueMemberSend = importedCreateQueue().add;
export const spreadJobs = { ...importedJobsNs };
export const constNsAlias = importedJobsNs;
export let mutableSend = (_row: { id: string }) => {};
mutableSend = getSender();
export const localCycleA: (row: { id: string }) => unknown = localCycleB;
export const localCycleB: (row: { id: string }) => unknown = localCycleA;
export const objHelpers = {
  toDto: (row: { id: string }) => row.id,
};

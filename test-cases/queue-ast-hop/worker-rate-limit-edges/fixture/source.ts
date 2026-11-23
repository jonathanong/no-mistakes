import { Worker } from "bullmq";

const innerWorker = new Worker("edge-jobs", async () => {});
const innerAlias = innerWorker;
export { innerAlias as renamedWorker };
export const unknownExport = getDynamic();

function getDynamic(): { rateLimit(ms: number): Promise<void> } {
  return { async rateLimit(_ms: number) {} };
}

export default function defaultHelper() {}

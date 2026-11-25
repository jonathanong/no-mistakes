import { Queue } from "bullmq";

export const MAIL_QUEUE = "mail";
export const ORPHAN_QUEUE = "orphan";
export const LOCAL_QUEUE = "local-loop";

export const mailQueue = new Queue(MAIL_QUEUE);
export const orphanQueue = new Queue(ORPHAN_QUEUE);
export const localQueue = new Queue(LOCAL_QUEUE);

export function createQueue(name: string, _opts?: { prefix?: string }) {
  return new Queue(name);
}

export function createWorker(
  _name: string,
  _processor: (job: { name: string }) => Promise<void>,
  _opts?: { prefix?: string },
) {}

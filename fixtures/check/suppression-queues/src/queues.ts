// no-mistakes-disable-file queues-check: unmatched topology is intentional
import { Queue, Worker } from 'bullmq';

export const queue = new Queue('lonely');
export const enqueue = () => queue.add('missing-worker', {});
export const worker = new Worker('lonely', async (job) => {
  if (job.name === 'missing-producer') return job.data;
});

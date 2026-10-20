import { createWorker } from '../lib.mts';
import { queue } from '../queues.mts';

async function handler(j) {
  await queue.add('mutated-shorthand', {}, { jobId: j.id });
}
// Shorthand destructuring also replaces the callable binding.
({ handler } = { handler: async () => {} });
createWorker('jobs', handler);

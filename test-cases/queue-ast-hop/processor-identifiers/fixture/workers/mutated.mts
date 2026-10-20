import { createWorker } from '../lib.mts';
import { queue } from '../queues.mts';

async function handler(j) {
  await queue.add('mutated', {}, { jobId: j.id });
}
// A reassigned declaration does not prove which body the worker will execute.
handler = async () => {};
createWorker('jobs', handler);

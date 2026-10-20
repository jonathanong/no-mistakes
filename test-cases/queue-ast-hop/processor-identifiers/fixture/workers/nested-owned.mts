import { createWorker } from '../lib.mts';
import { queue, other } from '../queues.mts';

createWorker('jobs', async (j) => {
  // The enclosing inline span is registered after these named processor spans.
  async function inner(k) {
    await queue.add('nested-wrong-queue', {}, { jobId: k.id });
  }
  createWorker('other', inner);
  async function own(m) {
    await other.add('nested-own-queue', {}, { jobId: m.id });
  }
  createWorker('other', own);
});

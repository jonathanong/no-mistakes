import { createWorker } from '../lib.mts';
import { queue } from '../queues.mts';

createWorker('jobs', async (j) => {
  const id = j.id;
  const copied = id;
  const dedup = j.deduplicationId;
  const unrelated = { id: 'different' };
  const outside = 'different';
  switch (j.name) {
    case 'copied-id': await queue.add('copied-id', {}, { jobId: copied }); break;
    case 'copied-dedup': await queue.add('copied-dedup', {}, { deduplication: { id: dedup } }); break;
    case 'unrelated-id': {
      const unrelatedId = unrelated.id;
      await queue.add('unrelated-id', {}, { jobId: unrelatedId });
      break;
    }
    case 'shadowed-id':
      [123].map((id) => queue.add('shadowed-id', {}, { jobId: id }));
      break;
    case 'shadowed-job':
      [unrelated].map((j) => queue.add('shadowed-job', {}, { jobId: j.id }));
      break;
    case 'block-leak': {
      // This alias expires with the inner block; the outer string survives.
      { const outside = j.id; }
      await queue.add('block-leak', {}, { jobId: outside });
      break;
    }
    case 'mutable-id': {
      let mutable = j.id;
      mutable = 'different';
      await queue.add('mutable-id', {}, { jobId: mutable });
    }
  }
});

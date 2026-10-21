import { createWorker } from '../lib.mts';
import { queue } from '../queues.mts';

declare const skipped: string;

createWorker('jobs', async (j) => {
  // `let` with no initializer, and a copy of the job object, are not id tokens.
  let bare;
  const alias = j;
  const {
    id,
    label,
    [skipped]: unused,
    extra: { nested },
    deduplicationId: dedup = 'fallback',
  } = j;
  // A computed key and a nested pattern are not id tokens.
  const { ['id']: computedId } = j;
  const {
    id: { nestedId },
  } = j;
  switch (j.name) {
    case 'destructured-id':
      await queue.add('destructured-id', {}, { jobId: id });
      break;
    case 'destructured-dedup':
      await queue.add('destructured-dedup', {}, { deduplication: { id: dedup } });
      break;
  }
  return { alias, label, unused, nested, bare, computedId, nestedId };
});

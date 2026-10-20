import { createWorker } from '../lib.mts';
import { queue } from '../queues.mts';

const handler = async (j) => {
  switch (j.name) {
    case 'scoped-token': await queue.add('scoped-token', {}, { jobId: j.id });
  }
};
function register() {
  createWorker('jobs', handler);
}
function shadowed() {
  // `var` belongs to this function even though its declaration is in a block.
  { var handler = async () => {}; }
  createWorker('other', handler);
}

// A shared processor does not establish one source queue for its enqueues.
const shared = async (j) => queue.add('shared-token', {}, { jobId: j.id });
createWorker('jobs', shared);
createWorker('other', shared);

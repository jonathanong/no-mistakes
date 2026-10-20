import { createWorker } from '../lib.mts';
import { queue, other } from '../queues.mts';

// The body precedes the worker invocation; its parameter is still the job.
async function handler(j) {
  switch (j.name) {
    case 'function-token':
      await queue.add('function-token', {}, { jobId: j.id });
      await other.add('return');
  }
}
createWorker('jobs', handler);

const arrow = async (j) => {
  const id = j.id;
  switch (j.name) {
    case 'arrow-token':
      await queue.add('arrow-token', {}, { deduplication: { id } });
  }
};
createWorker('jobs', arrow);

async function returnHandler(j) {
  switch (j.name) {
    case 'return': await queue.add('function-token');
  }
}
createWorker('other', returnHandler);

// A same-name parameter hides the outer callable and must not borrow its body.
function register(handler) {
  createWorker('other', handler);
}

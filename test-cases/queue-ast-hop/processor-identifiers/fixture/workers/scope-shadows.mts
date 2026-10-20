import { createWorker } from '../lib.mts';
import { queue } from '../queues.mts';

createWorker('jobs', async (j) => {
  const id = j.id;
  try {} catch {}
  try {} catch (id) {
    await queue.add('catch-id', {}, { jobId: id });
  }
  try {} catch (j) {
    await queue.add('catch-job', {}, { jobId: j.id });
  }
  {
    // Declaration names shadow the entire block, including earlier statements.
    await queue.add('hoisted-function', {}, { jobId: id });
    function id() {}
    await queue.add('function-id', {}, { jobId: id });
  }
  {
    await queue.add('hoisted-class', {}, { jobId: id });
    class id {}
    await queue.add('class-id', {}, { jobId: id });
  }
  {
    // Access before initialization is invalid at runtime; never borrow an outer token.
    await queue.add('hoisted-const', {}, { jobId: id });
    const id = 'different';
  }
  const Anonymous = class {};
  const expression = function id() {
    return queue.add('function-expression-id', {}, { jobId: id });
  };
  const Expression = class id {
    static enqueue() { return queue.add('class-expression-id', {}, { jobId: id }); }
  };
  // Leaving every shadow scope restores the copied active token.
  await queue.add('scope-restored', {}, { jobId: id });
});

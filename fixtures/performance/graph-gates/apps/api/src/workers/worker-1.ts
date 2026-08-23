import * as processors from '../processors/processors-1';
export const worker1 = new Worker('queue-1', async (job) => processors[job.name](job.data));

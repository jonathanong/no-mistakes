import * as processors from '../processors/processors-2.mts';
export const worker2 = new Worker('queue-2', async (job) => processors[job.name](job.data));

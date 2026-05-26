import * as processors from '../processors/processors-3.mts';
export const worker3 = new Worker('queue-3', async (job) => processors[job.name](job.data));

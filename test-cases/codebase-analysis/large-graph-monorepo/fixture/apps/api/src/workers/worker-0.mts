import * as processors from '../processors/processors-0.mts';
export const worker0 = new Worker('queue-0', async (job) => processors[job.name](job.data));

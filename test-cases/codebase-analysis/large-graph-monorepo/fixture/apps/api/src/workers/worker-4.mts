import * as processors from '../processors/processors-4.mts';
export const worker4 = new Worker('queue-4', async (job) => processors[job.name](job.data));

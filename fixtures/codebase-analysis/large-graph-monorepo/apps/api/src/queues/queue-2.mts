import { createQueue } from '@fixture/queue-factory';
export const queue2 = createQueue('queue-2', { concurrency: 3 });

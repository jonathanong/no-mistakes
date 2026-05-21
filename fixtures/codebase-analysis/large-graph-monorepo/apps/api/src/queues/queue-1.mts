import { createQueue } from '@fixture/queue-factory';
export const queue1 = createQueue('queue-1', { concurrency: 2 });

import { Worker } from 'glide-mq';
import { workerQueueConnection, workerQueuePrefix } from '@example/cache/glide-mq-client';
import { QUEUE_NAME } from './config.mts';

export const legacyWorkerGood = new Worker(
  QUEUE_NAME,
  async job => {
    return job.data;
  },
  {
    connection: workerQueueConnection,
    prefix: workerQueuePrefix,
    concurrency: 5,
  },
);

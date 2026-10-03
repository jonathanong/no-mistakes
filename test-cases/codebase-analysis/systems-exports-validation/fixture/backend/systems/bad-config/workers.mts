import { createWorker } from '@example/cache/glide-mq-factory';
import { QUEUE_NAME } from './config.mts';

export const badConfigWorker = createWorker(QUEUE_NAME, async job => job.data, {});

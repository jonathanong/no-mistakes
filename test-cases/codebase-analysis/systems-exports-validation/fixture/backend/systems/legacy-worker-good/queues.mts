import { createQueue } from '@example/cache/glide-mq-factory';
import { QUEUE_NAME } from './config.mts';

export const legacyWorkerGood = createQueue(QUEUE_NAME);

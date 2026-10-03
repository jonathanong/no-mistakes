import { createWorker } from '@example/cache/glide-mq-factory';

export const emailWorker = createWorker(emailQueue, processEmail, {
  lockDuration: 60_000,
});

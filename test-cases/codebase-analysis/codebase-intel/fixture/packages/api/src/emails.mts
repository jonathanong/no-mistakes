import { createQueue } from '@example/cache/glide-mq-factory';
export const emailsQueue = createQueue('emails', {});

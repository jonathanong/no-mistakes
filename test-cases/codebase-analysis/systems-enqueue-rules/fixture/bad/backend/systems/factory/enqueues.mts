import {
  createBulkEnqueueFunction,
  createEnqueueFunction,
} from '@example/cache/glide-mq-factory';
import { factoryQueue } from './queues.mts';

const enqueueSingleJob = createEnqueueFunction({
  queue: factoryQueue,
  queueName: 'factory',
  jobName: 'single',
});

const enqueueBulkJobs = createBulkEnqueueFunction({
  queue: factoryQueue,
  queueName: 'factory',
  jobName: 'bulk',
  buildJob: id => ({ data: { id } }),
});

export function enqueueBulkUsesSingleFactory(ids: string[]) {
  return enqueueSingleJob({ id: ids[0] });
}

export function enqueueSingleUsesBulkFactory(id: string) {
  return enqueueBulkJobs([id]);
}

import { createBulkEnqueueFunction, createEnqueueFunction } from '@example/cache/glide-mq-factory';
import { wrapperQueue } from './queues.mts';

function createSingleWrapper(jobName: string) {
  const enqueueJob = createEnqueueFunction({
    queue: wrapperQueue,
    queueName: 'wrappers',
    jobName,
  });

  return (id: string) => enqueueJob({ id });
}

function createBulkWrapper(jobName: string) {
  const enqueueBulkJobs = createBulkEnqueueFunction({
    queue: wrapperQueue,
    queueName: 'wrappers',
    jobName,
    buildJob: id => ({ data: { id } }),
  });

  return (ids: string[]) => enqueueBulkJobs(ids);
}

export const enqueueWrappedSingle = createSingleWrapper('wrappedSingle');
export const enqueueBulkWrapped = createBulkWrapper('wrappedBulk');

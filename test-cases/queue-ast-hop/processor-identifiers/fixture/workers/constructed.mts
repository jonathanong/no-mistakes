import { Worker } from 'bullmq';
import { queue } from '../queues.mts';

const processor = async (j) => {
  const id = j.id;
  switch (j.name) {
    case 'constructed-token': await queue.add('constructed-token', {}, { jobId: id });
  }
};
new Worker('jobs', processor);

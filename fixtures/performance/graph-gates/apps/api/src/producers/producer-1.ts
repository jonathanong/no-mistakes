import { queue1 } from '../queues/queue-1';
import { jobPayload1 } from '@fixture/jobs/job-1';
export function enqueue1_0() { return queue1.add('process1_0', jobPayload1()); }
export function enqueue1_1() { return queue1.add('process1_1', jobPayload1()); }
export function enqueueBulk1() { return queue1.addBulk([{ name: 'process1_0', data: {} }, { name: 'process1_1', data: {} }]); }

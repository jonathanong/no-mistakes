import { queue1 } from '../queues/queue-1.mts';
import { jobPayload1 } from '@fixture/jobs/job-1.mts';
export function enqueue1_0() { return queue1.add('process1_0', jobPayload1()); }
export function enqueue1_1() { return queue1.add('process1_1', jobPayload1()); }
export function enqueue1_2() { return queue1.add('process1_2', jobPayload1()); }
export function enqueue1_3() { return queue1.add('process1_3', jobPayload1()); }
export function enqueueBulk1() { return queue1.addBulk([{ name: 'process1_0', data: {} }, { name: 'process1_1', data: {} }]); }

import { queue2 } from '../queues/queue-2';
import { jobPayload2 } from '@fixture/jobs/job-2';
export function enqueue2_0() { return queue2.add('process2_0', jobPayload2()); }
export function enqueue2_1() { return queue2.add('process2_1', jobPayload2()); }
export function enqueueBulk2() { return queue2.addBulk([{ name: 'process2_0', data: {} }, { name: 'process2_1', data: {} }]); }

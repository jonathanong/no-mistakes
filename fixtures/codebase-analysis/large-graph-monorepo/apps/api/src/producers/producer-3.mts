import { queue3 } from '../queues/queue-3.mts';
import { jobPayload3 } from '@fixture/jobs/job-3.mts';
export function enqueue3_0() { return queue3.add('process3_0', jobPayload3()); }
export function enqueue3_1() { return queue3.add('process3_1', jobPayload3()); }
export function enqueue3_2() { return queue3.add('process3_2', jobPayload3()); }
export function enqueue3_3() { return queue3.add('process3_3', jobPayload3()); }
export function enqueueBulk3() { return queue3.addBulk([{ name: 'process3_0', data: {} }, { name: 'process3_1', data: {} }]); }

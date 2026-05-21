import { queue4 } from '../queues/queue-4.mts';
import { jobPayload4 } from '@fixture/jobs/job-4.mts';
export function enqueue4_0() { return queue4.add('process4_0', jobPayload4()); }
export function enqueue4_1() { return queue4.add('process4_1', jobPayload4()); }
export function enqueue4_2() { return queue4.add('process4_2', jobPayload4()); }
export function enqueue4_3() { return queue4.add('process4_3', jobPayload4()); }
export function enqueueBulk4() { return queue4.addBulk([{ name: 'process4_0', data: {} }, { name: 'process4_1', data: {} }]); }

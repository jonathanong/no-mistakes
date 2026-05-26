import { queue0 } from '../queues/queue-0.mts';
import { jobPayload0 } from '@fixture/jobs/job-0.mts';
export function enqueue0_0() { return queue0.add('process0_0', jobPayload0()); }
export function enqueue0_1() { return queue0.add('process0_1', jobPayload0()); }
export function enqueue0_2() { return queue0.add('process0_2', jobPayload0()); }
export function enqueue0_3() { return queue0.add('process0_3', jobPayload0()); }
export function enqueueBulk0() { return queue0.addBulk([{ name: 'process0_0', data: {} }, { name: 'process0_1', data: {} }]); }

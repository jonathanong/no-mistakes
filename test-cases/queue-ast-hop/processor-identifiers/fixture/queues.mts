import { createQueue } from './lib.mts';
export const queue = createQueue('jobs');
export const other = createQueue('other');

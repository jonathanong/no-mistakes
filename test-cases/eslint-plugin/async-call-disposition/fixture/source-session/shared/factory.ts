import { enqueue } from '@session/jobs';
type Job = typeof enqueue;
export function factory(): { run: Job } { return { run: enqueue }; }

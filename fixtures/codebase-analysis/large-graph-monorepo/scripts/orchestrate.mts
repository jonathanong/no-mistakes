import { exec, fork } from 'node:child_process';
exec('node scripts/worker-start.mts');
fork('scripts/api-start.mts');
export const orchestrated = true;

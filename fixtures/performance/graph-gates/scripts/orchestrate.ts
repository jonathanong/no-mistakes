import { exec, fork } from 'node:child_process';
exec('node scripts/worker-start.ts');
fork('scripts/api-start.ts');
export const orchestrated = true;

import { enqueue } from '@data/jobs';
import { start } from './pipeline';
const wrapped: typeof enqueue = () => enqueue();
Date.prototype.getTime = () => { wrapped(); return 0; };
void start({ enqueue: wrapped, now: new Date(0), compute: async () => 1 });

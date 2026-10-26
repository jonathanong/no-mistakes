import { enqueue } from '@data/jobs';
import { start } from './pipeline';
const wrapped: typeof enqueue = () => enqueue();
class Date { getTime() { wrapped(); return 0; } }
void start({ enqueue: wrapped, now: new Date() as any, compute: async () => 1 });

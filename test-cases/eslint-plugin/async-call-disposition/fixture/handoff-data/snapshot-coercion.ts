import { enqueue } from '@data/jobs';
import { start } from './pipeline';
const wrapped: typeof enqueue = () => enqueue();
const stamp = Date.parse({ toString() { [wrapped][0](); return '2025'; } } as unknown as string);
void start({ enqueue: wrapped, now: new Date(stamp), compute: async () => 1 });

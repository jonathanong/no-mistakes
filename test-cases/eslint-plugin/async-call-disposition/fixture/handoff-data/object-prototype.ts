import { enqueue } from '@data/jobs';
import { start } from './pipeline';
const wrapped: typeof enqueue = () => enqueue();
Object.defineProperty(Object.prototype, 'cursor', { get() { wrapped(); return undefined; } });
void start({ enqueue: wrapped, compute: async () => 1 });

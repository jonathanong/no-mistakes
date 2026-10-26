import { enqueue } from '@data/jobs';
import { start } from './pipeline';
const cutoffMs = Date.parse('2025-01-01');
const compute = async () => 1;
const wrapped: typeof enqueue = () => enqueue();
for (const [nowMs, count] of [[cutoffMs - 1, 1], [cutoffMs, 2], [cutoffMs + 1, 3]] as const) {
  void count;
  void start({ enqueue: wrapped, now: new Date(nowMs), compute });
}
const direct: typeof enqueue = () => enqueue();
void start({ enqueue: direct, now: new Date(Date.UTC(2025, 0, 1)), compute });
const absent: typeof enqueue = () => enqueue();
void start({ enqueue: absent, compute });
const unary: typeof enqueue = () => enqueue();
void start({ enqueue: unary, now: new Date(-1), 'compute': compute });

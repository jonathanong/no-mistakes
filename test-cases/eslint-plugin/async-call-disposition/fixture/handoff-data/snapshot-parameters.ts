import { enqueue } from '@data/jobs';
import { start } from './pipeline';
export function prior({ cutoff }: { cutoff: string }) {
  const stamp = Date.parse(cutoff);
  const wrapped: typeof enqueue = () => enqueue();
  const compute = async () => 1;
  for (const [nowMs] of [[stamp - 1], [stamp + 1]]) {
    void start({ enqueue: wrapped, now: new Date(nowMs), compute });
  }
}
export function later(cutoff: string) {
  const wrapped: typeof enqueue = () => enqueue();
  const stamp = Date.parse(cutoff);
  void start({ enqueue: wrapped, now: new Date(stamp), compute: async () => 1 });
}
export function hoisted(cutoff: string) {
  const stamp = Date.parse(cutoff);
  function wrapped() { return enqueue(); }
  void start({ enqueue: wrapped, now: new Date(stamp), compute: async () => 1 });
}
export function separateBlock(cutoff: string) {
  const stamp = Date.parse(cutoff);
  {
    const wrapped: typeof enqueue = () => enqueue();
    void start({ enqueue: wrapped, now: new Date(stamp), compute: async () => 1 });
  }
}
export function expressionSnapshot(cutoff: string) {
  const stamp = Date.parse(cutoff) + 1;
  const wrapped: typeof enqueue = () => enqueue();
  void start({ enqueue: wrapped, now: new Date(stamp), compute: async () => 1 });
}

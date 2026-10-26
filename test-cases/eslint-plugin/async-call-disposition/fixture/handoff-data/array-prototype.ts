import { enqueue } from '@data/jobs';
import { start } from './pipeline';
const wrapped: typeof enqueue = () => enqueue();
const arrays = Array.prototype;
// The source tuples do not determine the yielded values after iterator mutation.
arrays[Symbol.iterator] = function* () {
  yield { [Symbol.iterator]: function* () { yield { valueOf() { wrapped(); return 0; } }; } } as any;
};
for (const [nowMs] of [[0]]) {
  void start({ enqueue: wrapped, now: new Date(nowMs), compute: async () => 1 });
}

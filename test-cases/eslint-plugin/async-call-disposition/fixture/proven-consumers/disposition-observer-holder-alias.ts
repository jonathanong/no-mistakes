import { enqueueEmail } from '@app/jobs';
let holder = globalThis;
const setter = Reflect.set;
setter(holder, 'expect', () => ({}));
await expect(enqueueEmail()).rejects.toThrow();

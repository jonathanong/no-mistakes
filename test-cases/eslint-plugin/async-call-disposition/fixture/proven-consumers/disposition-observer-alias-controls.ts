import { enqueueEmail } from '@app/jobs';
async function readonlyAlias() {
  const p = enqueueEmail();
  const alias = p;
  p.catch(() => {});
}
let holder = globalThis;
const setter = Reflect.set;
setter(holder, 'fetch', () => {});
await expect(enqueueEmail()).rejects.toThrow();

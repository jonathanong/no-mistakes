import { expect as trustedExpect } from 'vitest';
import { enqueueEmail } from '@app/jobs';
globalThis.fetch = () => {};
Object.defineProperty(globalThis, 'fetch', { value: () => {} });
await expect(enqueueEmail()).rejects.toThrow();
await trustedExpect(enqueueEmail()).rejects.toThrow();
const unrelated = {};
Object.assign(unrelated, {});
Reflect.set(unrelated, unknownKey, () => {});
Object.defineProperty(); // Malformed reflection must not crash identity analysis.

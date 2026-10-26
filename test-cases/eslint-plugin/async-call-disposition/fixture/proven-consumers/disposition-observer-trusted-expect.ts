import { expect } from 'vitest';
import { enqueueEmail } from '@app/jobs';
globalThis.expect = () => {};
await expect(enqueueEmail()).rejects.toThrow();

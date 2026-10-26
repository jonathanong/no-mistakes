import { enqueueEmail } from '@app/jobs';
self.expect = () => ({});
await expect(enqueueEmail()).rejects.toThrow();

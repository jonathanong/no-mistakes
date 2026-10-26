import { enqueueEmail } from '@app/jobs';
global.expect = () => ({});
await expect(enqueueEmail()).rejects.toThrow();

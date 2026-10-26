import { enqueueEmail } from '@app/jobs';
window.expect = () => ({});
await expect(enqueueEmail()).rejects.toThrow();

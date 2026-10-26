import { enqueueEmail } from '@app/jobs';
async function cases() {
  await expect(enqueueEmail()).rejects.toThrow();
  const retained = enqueueEmail();
  await expect(retained).resolves.toBeDefined();
  expect(enqueueEmail()).rejects.toThrow();
}

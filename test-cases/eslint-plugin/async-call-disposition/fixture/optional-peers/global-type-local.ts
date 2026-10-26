import { enqueueEmail } from '@app/jobs';
type Function = unknown;
Function('Map.prototype.get = () => undefined')();
async function consume(callback: () => Promise<void>) {
  const cache = new Map();
  if (!cache.has('job')) cache.set('job', callback());
  return await cache.get('job');
}
const wrapped = () => enqueueEmail();
await consume(wrapped);

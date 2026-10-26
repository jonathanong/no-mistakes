import { type Function } from './global-types';
Function('Map.prototype.get = () => undefined')();
export async function consume(callback: () => Promise<void>) {
  const cache = new Map();
  if (!cache.has('job')) cache.set('job', callback());
  return await cache.get('job');
}

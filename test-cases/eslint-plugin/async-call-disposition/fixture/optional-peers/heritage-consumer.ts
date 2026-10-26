// Interface heritage is erased and must not escape the runtime Map constructor.
interface Unrelated extends Map<string, unknown> {}
export async function consume(callback: () => Promise<void>) {
  const cache = new Map();
  if (!cache.has('job')) cache.set('job', callback());
  return await cache.get('job');
}

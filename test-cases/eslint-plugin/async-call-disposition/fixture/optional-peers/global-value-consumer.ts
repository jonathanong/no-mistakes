const Function = (value: unknown) => value;
const globalThis = { marker: 1 };
function inspect() {
  type Function = string;
  type globalThis = unknown;
  Function('metadata');
  void globalThis.marker;
}
inspect();
export async function consume(callback: () => Promise<void>) {
  const cache = new Map();
  if (!cache.has('job')) cache.set('job', callback());
  return await cache.get('job');
}

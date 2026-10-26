export function drop(callback) { callback(); }
export function opaque(callback) { unknownConsumer(callback()); }
export function dropProperties(dependencies) { dependencies.enqueueEmail(); }
export async function shadowed(callback) {
  const consume = async (callback) => { await callback(); };
  await consume(async () => {});
  callback();
}
export async function defaultShadow(callback, other = callback()) {
  var callback = async () => {};
  await callback();
}

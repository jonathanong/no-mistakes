export async function alias(callback: () => Promise<void>) {
  await callback();
  { type callback = string; callback(); }
}
export async function generic(callback: () => Promise<void>) {
  await callback();
  function helper<callback>() { callback(); }
  helper();
}
export async function iface(callback: () => Promise<void>) {
  await callback();
  { interface callback {} callback(); }
}
export async function awaited(callback: () => Promise<void>) {
  await callback();
  { type callback = string; type Alias = callback; await callback(); }
}
export async function property(dependencies: { enqueueEmail: () => Promise<void> }) {
  await dependencies.enqueueEmail();
  { type dependencies = string; dependencies.enqueueEmail(); }
}
export async function propertyAwaited(dependencies: { enqueueEmail: () => Promise<void> }) {
  await dependencies.enqueueEmail();
  { type dependencies = string; await dependencies.enqueueEmail(); }
}

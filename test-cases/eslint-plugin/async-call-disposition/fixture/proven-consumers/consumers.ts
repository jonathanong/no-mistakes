export async function consume(callback: () => Promise<unknown>) {
  let result;
  result = await callback();
  return result;
}
export const consumeConst = async (callback: () => Promise<unknown>) => {
  return await callback();
};
export function floating(callback: () => Promise<unknown>) {
  callback();
}
export async function escaping(callback: () => Promise<unknown>) {
  await callback();
  unknownConsumer(callback);
}
export declare function ambient(callback: () => Promise<unknown>): Promise<unknown>;
declare function unknownConsumer(callback: unknown): void;
export async function evaluated(callback: () => Promise<unknown>) {
  eval("callback = async () => undefined");
  await callback();
}
export async function deferred(callback: () => Promise<unknown>) {
  return async () => await callback();
}
export let mutable = async (callback: () => Promise<unknown>) => await callback();
export async function exposed(callback: () => Promise<unknown>) { await callback(); }
unknownConsumer(exposed);
export async function argumentsRead(callback: () => Promise<unknown>) {
  void arguments;
  await callback();
}
export async function initialized(callback: () => Promise<unknown>, result = callback()) {
  await callback();
}
export async function replaced(callback: () => Promise<unknown>, result = (callback = async () => undefined)) {
  await callback();
}
export async function forwarded(callback: () => Promise<unknown>, result = unknownConsumer(callback)) {
  await callback();
}

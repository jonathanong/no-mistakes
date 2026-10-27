declare function retry(loader: () => Promise<unknown>): Promise<{ then(cb: (mod: any) => unknown): unknown }>;
declare function wrap(loader: () => Promise<unknown>): Promise<unknown>;

// The wrapper's result is chained, so the module is not necessarily its default.
export const retried = retry(() => import("./retried.mjs")).then((mod) => mod.Retried);
// The wrapper's result is awaited, so the module is not necessarily its default.
export async function wrapped() {
  return await wrap(() => import("./wrapped.mjs"));
}

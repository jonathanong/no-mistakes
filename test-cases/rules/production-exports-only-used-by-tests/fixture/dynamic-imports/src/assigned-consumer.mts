declare function retry(loader: () => Promise<unknown>): Promise<{ then(cb: (mod: any) => unknown): unknown }>;

// The stored wrapper result is chained later, so the advisory stays.
const stored = retry(() => import("./assigned-target.mjs"));
export const assigned = stored.then((mod: any) => mod.Assigned);

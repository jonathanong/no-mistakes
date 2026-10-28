declare function retry(loader: () => Promise<unknown>): Promise<{ then(cb: (mod: any) => unknown): unknown }>;

// Parentheses and `as` do not hide the chained result, so the advisory stays.
export const parenthesized = (retry(() => import("./paren-target.mjs")) as Promise<unknown>).then((mod: any) => mod.Paren);

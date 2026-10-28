declare function dynamic(loader: () => Promise<unknown>): unknown;
declare const k: string;

// An extra key, a spread or a computed key may read other names: advisories stay.
export const extraKey = dynamic(() => import("./then-default-extra.mjs").then((m) => ({ default: m.ExtraKey, other: 1 })));
export const spread = dynamic(() => import("./then-default-extra.mjs").then((m) => ({ ...m })));
export const computed = dynamic(() => import("./then-default-extra.mjs").then((m) => ({ [k]: m.Computed })));

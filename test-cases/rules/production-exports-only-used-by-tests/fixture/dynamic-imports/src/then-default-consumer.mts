declare function dynamic(loader: () => Promise<unknown>): unknown;

// Proven: the object's only property is `default` and it reads one named export.
export const expressionBody = dynamic(() => import("./then-default.mjs").then((m) => ({ default: m.ExprBody })));
export const blockBody = dynamic(() =>
  import("./then-default.mjs").then((m) => {
    return { default: m.BlockBody };
  }),
);
export const stringKey = dynamic(() => import("./then-default.mjs").then((m) => ({ "default": m.StringKey })));
export const bracketRead = dynamic(() => import("./then-default.mjs").then((m) => ({ default: m["BracketRead"] })));

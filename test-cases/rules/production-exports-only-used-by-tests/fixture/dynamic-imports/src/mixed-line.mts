// Imports carry no byte offsets, so an unproven import sharing a line and
// specifier with a proven one keeps the whole site unknown.
export const both = [import("./mixed-line-target.mjs").then((mod) => mod.shared), import("./mixed-line-target.mjs")];

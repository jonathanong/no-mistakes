const key = "unknownForm" as const;
// A computed member read is not provable, so the advisory must remain.
export const unknownLoaded = import("./unknown.mjs").then((mod) => mod[key]);

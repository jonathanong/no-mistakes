// A local alias named Pick is not the global utility type.
type Pick<T, K> = T;
export type Local = Pick<typeof import("../src/local-pick.mjs"), "localKey">;

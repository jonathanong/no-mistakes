declare namespace Utils {
  type Pick<T, K> = T;
}
export type Qualified = Utils.Pick<typeof import("../src/qualified-pick.mjs"), "qualKey">;

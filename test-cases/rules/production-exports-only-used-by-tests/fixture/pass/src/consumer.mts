import defaultUsed, { productionUsed } from "./mixed.mjs";
import type { Contract } from "./mixed.mjs";
export const result: Contract = { value: productionUsed() + defaultUsed() };

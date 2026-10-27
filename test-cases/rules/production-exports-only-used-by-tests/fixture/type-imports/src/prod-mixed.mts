// A mixed value and type import: both names are production consumers.
import { usedValue, type MixedType } from "./mixed.mjs";
export const mixed: MixedType = { id: String(usedValue) };

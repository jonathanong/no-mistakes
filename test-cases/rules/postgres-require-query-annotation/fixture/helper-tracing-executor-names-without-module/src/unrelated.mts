// Omitting importSpecifier must not treat this non-type import as a SQL root.
import { value } from "./unrelated-target.mts";
export const unused = value;

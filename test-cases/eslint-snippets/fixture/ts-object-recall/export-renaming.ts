export { original as renamed };
export { read as readOther } from "./db";
// The file proves the rename syntax, but not cross-file type semantics.
export type { Input as OtherInput } from "./types";
export * from "./db";

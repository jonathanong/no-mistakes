import type { SourceShape } from "./source.mts";

export type PublicShape = SourceShape;

export function helper() {
  type SourceShape = string;
  const value: SourceShape = "local";
  return value;
}

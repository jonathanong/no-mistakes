import type { SourceShape } from "./source.mts";

export type Box<T extends SourceShape> = T;

export interface HasValue<T extends SourceShape> {
  value: T;
}

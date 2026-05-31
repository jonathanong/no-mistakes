import type { SourceShape } from './source.mts';

export interface Derived extends SourceShape {
  value: SourceShape;
}

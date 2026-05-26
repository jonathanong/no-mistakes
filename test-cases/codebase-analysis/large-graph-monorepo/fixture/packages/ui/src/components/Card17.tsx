import { coreFn17 } from '@fixture/core/core-17.mts';
import type { DataRecord17 } from '@fixture/data/records/data-17.mts';
export function Card17({ record }: { record: DataRecord17 }) {
  return <section data-testid="card-17">{coreFn17()}{record.id}</section>;
}

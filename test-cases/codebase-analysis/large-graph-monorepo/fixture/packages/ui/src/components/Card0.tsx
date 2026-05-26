import { coreFn0 } from '@fixture/core/core-0.mts';
import type { DataRecord0 } from '@fixture/data/records/data-0.mts';
export function Card0({ record }: { record: DataRecord0 }) {
  return <section data-testid="card-0">{coreFn0()}{record.id}</section>;
}

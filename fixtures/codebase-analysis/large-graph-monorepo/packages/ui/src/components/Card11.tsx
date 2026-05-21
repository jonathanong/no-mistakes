import { coreFn11 } from '@fixture/core/core-11.mts';
import type { DataRecord11 } from '@fixture/data/records/data-11.mts';
export function Card11({ record }: { record: DataRecord11 }) {
  return <section data-testid="card-11">{coreFn11()}{record.id}</section>;
}

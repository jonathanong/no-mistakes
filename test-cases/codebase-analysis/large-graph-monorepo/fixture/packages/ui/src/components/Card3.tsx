import { coreFn3 } from '@fixture/core/core-3.mts';
import type { DataRecord3 } from '@fixture/data/records/data-3.mts';
export function Card3({ record }: { record: DataRecord3 }) {
  return <section data-testid="card-3">{coreFn3()}{record.id}</section>;
}

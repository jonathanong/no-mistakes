import { coreFn2 } from '@fixture/core/core-2.mts';
import type { DataRecord2 } from '@fixture/data/records/data-2.mts';
export function Card2({ record }: { record: DataRecord2 }) {
  return <section data-testid="card-2">{coreFn2()}{record.id}</section>;
}

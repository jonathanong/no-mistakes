import { coreFn2 } from '@fixture/core/core-2';
import type { DataRecord2 } from '@fixture/data/records/data-2';
export function Card2({ record }: { record: DataRecord2 }) {
  return <section data-testid="card-2" data-pw="card-2">{coreFn2()}{record.id}</section>;
}

import { coreFn7 } from '@fixture/core/core-7.mts';
import type { DataRecord7 } from '@fixture/data/records/data-7.mts';
export function Card7({ record }: { record: DataRecord7 }) {
  return <section data-testid="card-7">{coreFn7()}{record.id}</section>;
}

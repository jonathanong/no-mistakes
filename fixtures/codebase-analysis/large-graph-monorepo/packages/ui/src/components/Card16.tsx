import { coreFn16 } from '@fixture/core/core-16.mts';
import type { DataRecord16 } from '@fixture/data/records/data-16.mts';
export function Card16({ record }: { record: DataRecord16 }) {
  return <section data-testid="card-16">{coreFn16()}{record.id}</section>;
}

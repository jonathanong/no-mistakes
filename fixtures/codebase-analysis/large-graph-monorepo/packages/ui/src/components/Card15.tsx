import { coreFn15 } from '@fixture/core/core-15.mts';
import type { DataRecord15 } from '@fixture/data/records/data-15.mts';
export function Card15({ record }: { record: DataRecord15 }) {
  return <section data-testid="card-15">{coreFn15()}{record.id}</section>;
}

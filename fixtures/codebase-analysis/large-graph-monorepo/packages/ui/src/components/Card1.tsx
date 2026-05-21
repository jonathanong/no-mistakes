import { coreFn1 } from '@fixture/core/core-1.mts';
import type { DataRecord1 } from '@fixture/data/records/data-1.mts';
export function Card1({ record }: { record: DataRecord1 }) {
  return <section data-testid="card-1">{coreFn1()}{record.id}</section>;
}

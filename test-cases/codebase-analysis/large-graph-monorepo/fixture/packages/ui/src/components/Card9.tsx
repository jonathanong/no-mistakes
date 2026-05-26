import { coreFn9 } from '@fixture/core/core-9.mts';
import type { DataRecord9 } from '@fixture/data/records/data-9.mts';
export function Card9({ record }: { record: DataRecord9 }) {
  return <section data-testid="card-9">{coreFn9()}{record.id}</section>;
}

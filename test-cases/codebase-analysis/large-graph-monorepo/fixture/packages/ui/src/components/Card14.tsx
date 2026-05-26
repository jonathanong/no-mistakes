import { coreFn14 } from '@fixture/core/core-14.mts';
import type { DataRecord14 } from '@fixture/data/records/data-14.mts';
export function Card14({ record }: { record: DataRecord14 }) {
  return <section data-testid="card-14">{coreFn14()}{record.id}</section>;
}

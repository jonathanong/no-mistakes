import { coreFn13 } from '@fixture/core/core-13.mts';
import type { DataRecord13 } from '@fixture/data/records/data-13.mts';
export function Card13({ record }: { record: DataRecord13 }) {
  return <section data-testid="card-13">{coreFn13()}{record.id}</section>;
}

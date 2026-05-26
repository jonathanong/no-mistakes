import { coreFn5 } from '@fixture/core/core-5.mts';
import type { DataRecord5 } from '@fixture/data/records/data-5.mts';
export function Card5({ record }: { record: DataRecord5 }) {
  return <section data-testid="card-5">{coreFn5()}{record.id}</section>;
}

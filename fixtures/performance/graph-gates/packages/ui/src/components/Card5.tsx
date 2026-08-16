import { coreFn5 } from '@fixture/core/core-5';
import type { DataRecord5 } from '@fixture/data/records/data-5';
export function Card5({ record }: { record: DataRecord5 }) {
  return <section data-testid="card-5" data-pw="card-5">{coreFn5()}{record.id}</section>;
}

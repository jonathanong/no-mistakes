import { coreFn10 } from '@fixture/core/core-10.mts';
import type { DataRecord10 } from '@fixture/data/records/data-10.mts';
export function Card10({ record }: { record: DataRecord10 }) {
  return <section data-testid="card-10">{coreFn10()}{record.id}</section>;
}

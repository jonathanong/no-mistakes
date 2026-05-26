import { coreFn6 } from '@fixture/core/core-6.mts';
import type { DataRecord6 } from '@fixture/data/records/data-6.mts';
export function Card6({ record }: { record: DataRecord6 }) {
  return <section data-testid="card-6">{coreFn6()}{record.id}</section>;
}

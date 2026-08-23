import { coreFn4 } from '@fixture/core/core-4';
import type { DataRecord4 } from '@fixture/data/records/data-4';
export function Card4({ record }: { record: DataRecord4 }) {
  return <section data-testid="card-4" data-pw="card-4">{coreFn4()}{record.id}</section>;
}

import { coreFn0 } from '@fixture/core/core-0';
import type { DataRecord0 } from '@fixture/data/records/data-0';
export function Card0({ record }: { record: DataRecord0 }) {
  return <section data-testid="card-0" data-pw="card-0">{coreFn0()}{record.id}</section>;
}
